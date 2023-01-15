#[cfg(test)]
#[cfg(feature = "it")]
mod tests {
    use std::collections::vec_deque::VecDeque;
    use std::path::Path;

    use anyhow::Result;
    use encoding_rs::UTF_8;
    use log::LevelFilter;

    use libakaza::engine::base::HenkanEngine;
    use libakaza::engine::bigram_word_viterbi_engine::{
        BigramWordViterbiEngine, BigramWordViterbiEngineBuilder,
    };
    use libakaza::graph::graph_resolver::Candidate;
    use libakaza::lm::system_bigram::MarisaSystemBigramLM;
    use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;
    use libakaza::skk::skkdict::read_skkdict;

    fn load_akaza() -> Result<BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM>>
    {
        let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/../akaza-data/data/";
        assert!(Path::new(datadir.as_str()).exists());
        BigramWordViterbiEngineBuilder::new(
            datadir.as_str(),
            Some(read_skkdict(
                Path::new(
                    (env!("CARGO_MANIFEST_DIR").to_string()
                        + "/../akaza-data/data/SKK-JISYO.akaza")
                        .as_str(),
                ),
                UTF_8,
            )?),
            Some(read_skkdict(
                Path::new(
                    (env!("CARGO_MANIFEST_DIR").to_string()
                        + "/../akaza-data/skk-dev-dict/SKK-JISYO.emoji")
                        .as_str(),
                ),
                UTF_8,
            )?),
        )
        .build()
    }

    struct Tester {
        akaza: BigramWordViterbiEngine<MarisaSystemUnigramLM, MarisaSystemBigramLM>,
    }

    impl Tester {
        fn new() -> Result<Tester> {
            Ok(Tester {
                akaza: load_akaza()?,
            })
        }

        fn test(&self, yomi: &str, kanji: &str) -> Result<()> {
            let got1 = &self.akaza.convert(yomi, None)?;
            let terms: Vec<String> = got1.iter().map(|f| f[0].kanji.clone()).collect();
            let got = terms.join("");
            assert_eq!(got, kanji);
            Ok(())
        }
    }

    #[test]
    fn test_go() -> Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let yomi = "ご";
        let got: Vec<VecDeque<Candidate>> = load_akaza()?.convert(yomi, None)?;
        assert_eq!(&got[0][0].yomi, "ご");
        let words: Vec<String> = got[0].iter().map(|x| x.kanji.to_string()).collect();
        assert!(words.contains(&"語".to_string()));
        // assert_eq!(got, kanji);
        Ok(())
    }

    #[test]
    fn test_sushi() -> Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let yomi = "すし";
        let got: Vec<VecDeque<Candidate>> = load_akaza()?.convert(yomi, None)?;
        assert_eq!(&got[0][0].yomi, "すし");
        let words: Vec<String> = got[0].iter().map(|x| x.kanji.to_string()).collect();
        assert!(words.contains(&"🍣".to_string()));
        // assert_eq!(got, kanji);
        Ok(())
    }

    #[test]
    fn test_with_data() -> anyhow::Result<()> {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Trace)
            .is_test(true)
            .try_init();
        let data: Vec<(&str, &str)> = vec![
            ("わたしのなまえはなかのです", "私の名前は中野です"),
            ("きめつのやいば", "鬼滅の刃"),
            ("かたがわいっしゃせん", "片側一車線"),
            ("みかくにん", "未確認"),
            ("はくしかてい", "博士課程"),
            ("にほん", "日本"),
            ("にっぽん", "日本"),
            ("http://mixi.jp", "http://mixi.jp"),
            ("https://mixi.jp", "https://mixi.jp"),
            ("nisitemo,", "にしても、"),
            (
                "けいやくないようをめいかくにするいぎ",
                "契約内容を明確にする意義",
            ),
            (
                "ろうどうしゃさいがいほしょうほけんほう",
                "労働者災害補償保険法",
            ),
            ("けいやくのしゅたいとは", "契約の主体とは"),
            ("tanosiijikan", "楽しい時間"),
            ("たのしいじかん", "楽しい時間"),
            ("zh", "←"),
            ("watasinonamaehanakanodesu.", "私の名前は中野です。"),
            ("わたしのなまえはなかのです。", "私の名前は中野です。"),
            ("わーど", "ワード"),
            ("にほん", "日本"),
            ("にっぽん", "日本"),
            ("siinn", "子音"),
            ("IME", "IME"),
            ("ややこしい", "ややこしい"),
            ("むずかしくない", "難しくない"),
            ("きぞん", "既存"),
            ("のぞましい", "望ましい"),
            ("こういう", "こういう"),
            ("はやくち", "早口"),
            ("しょうがっこう", "小学校"),
            ("げすとだけ", "ゲストだけ"),
            ("ぜんぶでてるやつ", "全部でてる奴"),
            ("えらべる", "選べる"),
            ("わたしだよ", "私だよ"),
            ("にほんごじょうほう", "日本語情報"),
            // ("れいわ", "令和"),
            ("ちいさい", "小さい"),
            // ↓現状のロジックでうまく変換できないもの。
            // ("かりきゅれーたー", "カリキュレーター"),
            // ("いたいのいたいのとんでけー", "痛いの痛いのとんでけー"),
            // ("そうみたいですね", "そうみたいですね"),
            // ("それなwww", "それなwww"),
            // ("sorenawww", "それなwww"),
        ];
        let tester = Tester::new()?;
        for (yomi, surface) in data {
            tester.test(yomi, surface)?;
        }
        Ok(())
    }
}
