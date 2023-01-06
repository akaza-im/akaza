#[cfg(test)]
mod tests {
    use std::collections::vec_deque::VecDeque;

    use anyhow::Result;

    use libakaza::akaza_builder::{Akaza, AkazaBuilder};
    use libakaza::graph::graph_resolver::Candidate;

    fn load_akaza() -> Result<Akaza> {
        let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/../akaza-data/data/";
        AkazaBuilder::default()
            .system_data_dir(datadir.as_str())
            .build()
    }

    struct Tester {
        akaza: Akaza,
    }

    impl Tester {
        fn new() -> Result<Tester> {
            Ok(Tester {
                akaza: load_akaza()?,
            })
        }

        fn test(&self, yomi: &str, kanji: &str) -> Result<()> {
            let got1 = &self.akaza.convert(yomi, &Vec::new())?;
            let terms: Vec<String> = got1.iter().map(|f| f[0].kanji.clone()).collect();
            let got = terms.join("");
            assert_eq!(got, kanji);
            Ok(())
        }
    }

    #[test]
    fn test_sushi() -> Result<()> {
        env_logger::builder().is_test(true).try_init()?;

        let yomi = "すし";
        let got: Vec<VecDeque<Candidate>> = load_akaza()?.convert(yomi, &vec![])?;
        assert_eq!(&got[0][0].yomi, "すし");
        let words: Vec<String> = got[0].iter().map(|x| x.kanji.to_string()).collect();
        assert!(words.contains(&"🍣".to_string()));
        // assert_eq!(got, kanji);
        Ok(())
    }

    #[test]
    fn test_with_data() -> anyhow::Result<()> {
        let data: Vec<(&str, &str)> = vec![
            ("わたしのなまえはなかのです", "私の名前は中野です"),
            ("かたがわいっしゃせん", "片側一車線"),
            ("みかくにん", "未確認"),
            ("きめつのやいば", "鬼滅の刃"),
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
            ("むずかしくない", "難しく無い"),
            ("きぞん", "既存"),
            ("のぞましい", "望ましい"),
            ("こういう", "こういう"),
            ("はやくち", "早口"),
            ("しょうがっこう", "小学校"),
            ("げすとだけ", "ゲストだけ"),
            ("ぜんぶでてるやつ", "全部でてるやつ"),
            ("えらべる", "選べる"),
            ("わたしだよ", "わたしだよ"),
            ("にほんごじょうほう", "日本語情報"),
            ("れいわ", "令和"),
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
