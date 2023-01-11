#[cfg(test)]
#[cfg(feature = "it")]
mod tests {
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::{BufReader, Read};

    use encoding_rs::EUC_JP;
    use kelp::{hira2kata, ConvOption};
    use regex::Regex;

    use libakaza::kana_kanji_dict::KanaKanjiDict;
    use libakaza::skk::skkdict::parse_skkdict;

    /// そうは読まないでしょ、というような読み方のものをいくつか登録しておく。
    /// (このテストは kytea が読み間違えなくなったら通る)
    #[test]
    #[ignore]
    fn test() -> anyhow::Result<()> {
        let dict = KanaKanjiDict::load("data/system_dict.trie")?;
        let ku = dict.find("く").unwrap();

        assert!(
            !ku.contains(&"薬".to_string()),
            "薬という字は「く」とは読まない: {:?}",
            ku
        );
        Ok(())
    }

    /// 1文字の漢字は変換速度に悪影響を与えるのでできるだけ削りたい。
    #[test]
    fn test_1moji_kanji() -> anyhow::Result<()> {
        let dict = KanaKanjiDict::load("data/system_dict.trie")?;

        // SKK-JISYO.L を読み込む
        let file = File::open("skk-dev-dict/SKK-JISYO.L")?;
        let mut buf: Vec<u8> = Vec::new();
        BufReader::new(file).read_to_end(&mut buf)?;
        let (p, _, _) = EUC_JP.decode(buf.as_slice());
        let (_, nasi) = parse_skkdict(p.to_string().as_str())?;

        // システムかな漢字辞書に、1文字で登録されているものをリストアップする。
        let single_char_yomis: Vec<String> = dict
            .all_yomis()
            .unwrap()
            .iter()
            .filter(|x| x.chars().count() == 1)
            .map(|s| s.to_string())
            .collect();

        let skk_comment_pattern = Regex::new(";.*").unwrap();

        for moji in &single_char_yomis {
            // SKKの辞書に登録されているもの、平仮名そのもの、カタカナそのものを登録する。
            let mut known_words: HashSet<String> = HashSet::new();

            if let Some(got) = nasi.get(moji) {
                got.iter()
                    .map(|x| skk_comment_pattern.replace(x, "").to_string())
                    .for_each(|it| {
                        known_words.insert(it);
                    });
            }
            known_words.insert(moji.to_string());
            known_words.insert(hira2kata(moji.as_str(), ConvOption::default()));

            // それら以外のものをリストアップする。
            let system_dict_only: Vec<String> = dict
                .find(moji)
                .unwrap()
                .iter()
                .filter(|p| !known_words.contains(*p))
                .map(|it| it.to_string())
                .collect();
            if !system_dict_only.is_empty() {
                println!("moji={:?}, system_dict_len={:?}", moji, system_dict_only);
            }
        }
        Ok(())
    }
}
