use std::time::SystemTime;

use log::info;

use libakaza::kana_kanji_dict::KanaKanjiDict;
use libakaza::kana_trie::marisa_kana_trie::MarisaKanaTrie;

pub fn make_kana_trie(system_dict_file: &str, kana_trie_file: &str) -> anyhow::Result<()> {
    let t1 = SystemTime::now();

    let system_kana_kanji_dict = KanaKanjiDict::load(system_dict_file)?;
    let all_yomis = system_kana_kanji_dict.all_yomis().unwrap();
    let system_kana_trie = MarisaKanaTrie::build(all_yomis);

    let t2 = SystemTime::now();

    info!(
        "Built kana-trie in {} msec",
        t2.duration_since(t1)?.as_millis()
    );

    system_kana_trie.save(kana_trie_file)?;

    let t3 = SystemTime::now();

    info!(
        "Saved kana-trie in {} msec",
        t3.duration_since(t2)?.as_millis()
    );

    Ok(())
}
