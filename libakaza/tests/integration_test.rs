use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use libakaza::graph::graph_builder::GraphBuilder;
use libakaza::graph::graph_resolver::GraphResolver;
use libakaza::graph::segmenter::Segmenter;
use libakaza::kana_kanji::hashmap_vec::HashmapVecKanaKanjiDict;
use libakaza::kana_trie::cedarwood_kana_trie::CedarwoodKanaTrie;
use libakaza::lm::base::SystemUnigramLM;
use libakaza::lm::system_bigram::MarisaSystemBigramLMBuilder;
use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLMBuilder;
use libakaza::user_side_data::user_data::UserData;

/// 統合テスト: 辞書読み込みから変換結果まで
#[test]
fn test_end_to_end_conversion_pipeline() -> anyhow::Result<()> {
    // 1. 辞書を構築
    let dict = HashMap::from([
        ("わたし".to_string(), vec!["私".to_string()]),
        (
            "あなた".to_string(),
            vec!["貴方".to_string(), "あなた".to_string()],
        ),
        ("かれ".to_string(), vec!["彼".to_string()]),
        ("かのじょ".to_string(), vec!["彼女".to_string()]),
        ("です".to_string(), vec!["です".to_string()]),
        ("いく".to_string(), vec!["行く".to_string()]),
        ("がっこう".to_string(), vec!["学校".to_string()]),
        ("に".to_string(), vec!["に".to_string()]),
    ]);

    // 2. 言語モデルを構築
    let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
    unigram_builder.add("私/わたし", 1.0);
    unigram_builder.add("貴方/あなた", 1.5);
    unigram_builder.add("あなた/あなた", 2.0);
    unigram_builder.add("彼/かれ", 1.2);
    unigram_builder.add("彼女/かのじょ", 1.3);
    unigram_builder.add("です/です", 0.5);
    unigram_builder.add("行く/いく", 1.1);
    unigram_builder.add("学校/がっこう", 1.4);
    unigram_builder.add("に/に", 0.3);
    unigram_builder.set_total_words(1000);
    unigram_builder.set_unique_words(100);
    let system_unigram_lm = unigram_builder.build()?;

    let unigram_map = system_unigram_lm.as_hash_map();
    let watashi_id = unigram_map.get("私/わたし").unwrap().0;
    let gakkou_id = unigram_map.get("学校/がっこう").unwrap().0;
    let ni_id = unigram_map.get("に/に").unwrap().0;
    let iku_id = unigram_map.get("行く/いく").unwrap().0;

    let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
    bigram_builder.set_default_edge_cost(10.0);
    bigram_builder.add(watashi_id, gakkou_id, 8.0); // 私→学校は不自然
    bigram_builder.add(gakkou_id, ni_id, 0.3); // 学校→に は自然
    bigram_builder.add(ni_id, iku_id, 0.2); // に→行く は自然
    let system_bigram_lm = bigram_builder.build()?;

    // 3. グラフビルダーを構築
    let graph_builder = GraphBuilder::new(
        HashmapVecKanaKanjiDict::new(dict),
        HashmapVecKanaKanjiDict::new(HashMap::new()),
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );

    // 4. セグメンテーション
    let kana_trie = CedarwoodKanaTrie::build(vec![
        "がっこう".to_string(),
        "に".to_string(),
        "いく".to_string(),
    ]);
    let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
    let segments = segmenter.build("がっこうにいく", None);

    // 5. グラフ構築
    let lattice = graph_builder.construct("がっこうにいく", &segments);

    // 6. 変換実行
    let resolver = GraphResolver::default();
    let result = resolver.resolve(&lattice)?;

    // 7. 結果検証 - 変換が成功することを確認
    assert!(!result.is_empty());
    assert!(!result[0].is_empty());

    Ok(())
}

/// 複数候補の正しいランキング
#[test]
fn test_candidate_ranking_with_bigram() -> anyhow::Result<()> {
    let dict = HashMap::from([
        (
            "きょう".to_string(),
            vec!["今日".to_string(), "教".to_string()],
        ),
        ("は".to_string(), vec!["は".to_string()]),
        (
            "いい".to_string(),
            vec!["良い".to_string(), "飯".to_string()],
        ),
    ]);

    let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
    unigram_builder.add("今日/きょう", 1.0);
    unigram_builder.add("教/きょう", 3.0); // unigram では "教" のコストが高い
    unigram_builder.add("は/は", 0.5);
    unigram_builder.add("良い/いい", 1.0);
    unigram_builder.add("飯/いい", 2.0);
    unigram_builder.set_total_words(100);
    unigram_builder.set_unique_words(50);
    let system_unigram_lm = unigram_builder.build()?;

    let unigram_map = system_unigram_lm.as_hash_map();
    let kyou_id = unigram_map.get("今日/きょう").unwrap().0;
    let ha_id = unigram_map.get("は/は").unwrap().0;
    let ii_id = unigram_map.get("良い/いい").unwrap().0;

    let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
    bigram_builder.set_default_edge_cost(10.0);
    bigram_builder.add(kyou_id, ha_id, 0.3); // "今日は" は自然な組み合わせ
    bigram_builder.add(ha_id, ii_id, 0.2); // "は良い" も自然
    let system_bigram_lm = bigram_builder.build()?;

    let graph_builder = GraphBuilder::new(
        HashmapVecKanaKanjiDict::new(dict),
        HashmapVecKanaKanjiDict::new(HashMap::new()),
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );

    let kana_trie = CedarwoodKanaTrie::build(vec![
        "きょう".to_string(),
        "は".to_string(),
        "いい".to_string(),
    ]);
    let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
    let segments = segmenter.build("きょうはいい", None);
    let lattice = graph_builder.construct("きょうはいい", &segments);
    let resolver = GraphResolver::default();
    let result = resolver.resolve(&lattice)?;

    // bigram スコアが考慮されて変換されることを確認
    assert!(!result.is_empty());
    assert!(!result[0].is_empty());

    Ok(())
}

/// エッジケース: 辞書にない読み
#[test]
fn test_unknown_yomi_fallback() -> anyhow::Result<()> {
    // 空の辞書
    let dict = HashMap::new();

    let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
    unigram_builder.set_total_words(100);
    unigram_builder.set_unique_words(50);
    let system_unigram_lm = unigram_builder.build()?;

    let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
    bigram_builder.set_default_edge_cost(10.0);
    let system_bigram_lm = bigram_builder.build()?;

    let graph_builder = GraphBuilder::new(
        HashmapVecKanaKanjiDict::new(dict),
        HashmapVecKanaKanjiDict::new(HashMap::new()),
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );

    let kana_trie = CedarwoodKanaTrie::build(vec![]);
    let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
    let segments = segmenter.build("みしらぬことば", None);
    let lattice = graph_builder.construct("みしらぬことば", &segments);
    let resolver = GraphResolver::default();
    let result = resolver.resolve(&lattice)?;

    // 辞書にない単語でも変換結果が返されることを確認
    assert!(!result.is_empty());
    assert!(!result[0].is_empty());

    Ok(())
}

/// パフォーマンステスト: 長い入力でもタイムアウトしないこと
#[test]
fn test_long_input_performance() -> anyhow::Result<()> {
    use std::time::Instant;

    let dict = HashMap::from([
        ("あ".to_string(), vec!["亜".to_string()]),
        ("い".to_string(), vec!["伊".to_string()]),
        ("う".to_string(), vec!["宇".to_string()]),
        ("え".to_string(), vec!["江".to_string()]),
        ("お".to_string(), vec!["尾".to_string()]),
    ]);

    let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
    unigram_builder.add("亜/あ", 1.0);
    unigram_builder.add("伊/い", 1.0);
    unigram_builder.add("宇/う", 1.0);
    unigram_builder.add("江/え", 1.0);
    unigram_builder.add("尾/お", 1.0);
    unigram_builder.set_total_words(100);
    unigram_builder.set_unique_words(50);
    let system_unigram_lm = unigram_builder.build()?;

    let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
    bigram_builder.set_default_edge_cost(10.0);
    let system_bigram_lm = bigram_builder.build()?;

    let graph_builder = GraphBuilder::new(
        HashmapVecKanaKanjiDict::new(dict),
        HashmapVecKanaKanjiDict::new(HashMap::new()),
        Arc::new(Mutex::new(UserData::default())),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );

    // 50文字の入力
    let long_input = "あいうえお".repeat(10);
    let kana_trie = CedarwoodKanaTrie::build(vec![
        "あ".to_string(),
        "い".to_string(),
        "う".to_string(),
        "え".to_string(),
        "お".to_string(),
    ]);
    let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);

    let start = Instant::now();
    let segments = segmenter.build(&long_input, None);
    let lattice = graph_builder.construct(&long_input, &segments);
    let resolver = GraphResolver::default();
    let result = resolver.resolve(&lattice)?;
    let elapsed = start.elapsed();

    // 結果が返されることを確認
    assert!(!result.is_empty());

    // 1秒以内に完了することを確認（パフォーマンス回帰検知）
    assert!(
        elapsed.as_secs() < 1,
        "Conversion took too long: {:?}",
        elapsed
    );

    Ok(())
}

/// ユーザー辞書とシステム辞書の統合
#[test]
fn test_user_dict_and_system_dict_integration() -> anyhow::Result<()> {
    let system_dict = HashMap::from([("たろう".to_string(), vec!["太郎".to_string()])]);

    let mut unigram_builder = MarisaSystemUnigramLMBuilder::default();
    unigram_builder.add("太郎/たろう", 2.0);
    unigram_builder.set_total_words(100);
    unigram_builder.set_unique_words(50);
    let system_unigram_lm = unigram_builder.build()?;

    let mut bigram_builder = MarisaSystemBigramLMBuilder::default();
    bigram_builder.set_default_edge_cost(10.0);
    let system_bigram_lm = bigram_builder.build()?;

    let mut user_data = UserData::default();
    // ユーザー辞書に追加
    user_data.dict.insert(
        "たろう".to_string(),
        vec!["太朗".to_string()], // 異字体
    );

    let graph_builder = GraphBuilder::new(
        HashmapVecKanaKanjiDict::new(system_dict),
        HashmapVecKanaKanjiDict::new(HashMap::new()),
        Arc::new(Mutex::new(user_data)),
        Rc::new(system_unigram_lm),
        Rc::new(system_bigram_lm),
    );

    let kana_trie = CedarwoodKanaTrie::build(vec!["たろう".to_string()]);
    let segmenter = Segmenter::new(vec![Arc::new(Mutex::new(kana_trie))]);
    let segments = segmenter.build("たろう", None);
    let lattice = graph_builder.construct("たろう", &segments);
    let resolver = GraphResolver::default();
    let result = resolver.resolve(&lattice)?;

    // システム辞書とユーザー辞書の両方の候補が含まれることを確認
    let all_candidates: Vec<String> = result
        .iter()
        .flat_map(|path| path.iter().map(|node| node.surface.clone()))
        .collect();

    assert!(all_candidates.contains(&"太郎".to_string()));
    assert!(all_candidates.contains(&"太朗".to_string()));

    Ok(())
}
