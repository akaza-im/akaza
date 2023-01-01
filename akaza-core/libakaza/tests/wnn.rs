#[cfg(test)]
mod tests {
    use std::collections::vec_deque::VecDeque;

    use anyhow::Result;

    use libakaza::akaza_builder::{Akaza, AkazaBuilder};
    use libakaza::graph::graph_resolver::Candidate;

    fn load_akaza() -> Result<Akaza> {
        let datadir = env!("CARGO_MANIFEST_DIR").to_string() + "/../../akaza-data/data/";
        AkazaBuilder::default()
            .system_data_dir(datadir.as_str())
            .build()
    }

    fn test(yomi: &str, kanji: &str) -> Result<()> {
        let got = load_akaza()?.convert_to_string(yomi)?;
        assert_eq!(got, kanji);
        Ok(())
    }

    #[test]
    fn test_wnn() -> Result<()> {
        test("わたしのなまえはなかのです", "私の名前は中野です")
    }

    #[test]
    fn test_working() -> Result<()> {
        test(
            "ろうどうしゃさいがいほしょうほけんほう",
            "労働者災害補償保険法",
        )
    }

    #[test]
    fn test_sushi() -> Result<()> {
        env_logger::builder().is_test(true).try_init()?;

        let yomi = "すし";
        let got: Vec<VecDeque<Candidate>> = load_akaza()?.convert(yomi)?;
        assert_eq!(&got[0][0].yomi, "すし");
        let words: Vec<String> = got[0].iter().map(|x| x.kanji.to_string()).collect();
        assert!(words.contains(&"🍣".to_string()));
        // assert_eq!(got, kanji);
        Ok(())
    }

    #[test]
    fn test_with_data() -> anyhow::Result<()> {
        let data: Vec<(&str, &str)> = vec![
            ("かたがわいっしゃせん", "片側一車線"),
            ("みかくにん", "未確認"),
            ("きめつのやいば", "鬼滅の刃"),
            ("はくしかてい", "博士課程"),
            ("にほん", "日本"),
            ("にっぽん", "日本"),
            // ↓現状のロジックでうまく変換できないもの。
            // ("かりきゅれーたー", "カリキュレーター"),
            // ("いたいのいたいのとんでけー", "痛いの痛いのとんでけー"),
        ];
        for (yomi, surface) in data {
            test(yomi, surface)?;
        }
        Ok(())
    }
}
