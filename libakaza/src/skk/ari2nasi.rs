use std::collections::HashMap;

use anyhow::bail;

use crate::romkan::RomKanConverter;

const BOIN: [char; 5] = ['a', 'i', 'u', 'e', 'o'];

pub struct Ari2Nasi {
    romkan_converter: RomKanConverter,
}

impl Ari2Nasi {
    pub fn new(romkan_converter: RomKanConverter) -> Ari2Nasi {
        Ari2Nasi { romkan_converter }
    }

    fn expand_okuri(
        &self,
        kana: &str,
        kanjis: &Vec<String>,
    ) -> anyhow::Result<Vec<(String, Vec<String>)>> {
        let Some(last_char) = kana.chars().last() else {
            bail!("kana is empty");
        };
        if last_char.is_ascii_alphabetic() {
            if BOIN.contains(&last_char) {
                // 母音の場合はそのまま平仮名に変換する。
                // e.g. "a" → "あ"
                let okuri = self
                    .romkan_converter
                    .to_hiragana(last_char.to_string().as_str());
                let yomi = &kana[0..kana.len() - last_char.len_utf8()];
                let kanjis = kanjis
                    .iter()
                    .map(|f| f.to_string() + okuri.as_str())
                    .collect();
                Ok(vec![(yomi.to_string(), kanjis)])
            } else {
                // 子音の場合は母音の組み合わせによって全パターンつくって返す。
                let mut result: Vec<(String, Vec<String>)> = Vec::new();
                let yomi_base = &kana[0..kana.len() - last_char.len_utf8()].to_string();
                for b in BOIN {
                    let okuri = self
                        .romkan_converter
                        .to_hiragana((last_char.to_string() + b.to_string().as_str()).as_str());
                    if okuri.chars().filter(|c| c.is_ascii_alphabetic()).count() > 0 {
                        // "wu" のような、平仮名に変換できない不正なローマ字パターンを生成しているケースもある。
                        // そういう場合は、スキップ。
                        continue;
                    }
                    let kanjis = kanjis
                        .iter()
                        .map(|f| f.to_string() + okuri.as_str())
                        .collect();
                    result.push((yomi_base.to_string() + okuri.to_string().as_str(), kanjis));
                }
                Ok(result)
            }
        } else {
            Ok(vec![(
                kana.to_string(),
                kanjis.iter().map(|f| f.to_string()).collect(),
            )])
        }
    }

    pub fn ari2nasi(
        &self,
        src: &HashMap<String, Vec<String>>,
    ) -> anyhow::Result<HashMap<String, Vec<String>>> {
        let mut retval: HashMap<String, Vec<String>> = HashMap::new();
        for (kana, kanjis) in src.iter() {
            for (kkk, vvv) in self.expand_okuri(kana, kanjis)? {
                retval.insert(kkk, vvv);
            }
        }
        Ok(retval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_okuri() -> anyhow::Result<()> {
        let ari2nasi = Ari2Nasi::new(RomKanConverter::default());
        let got = ari2nasi.expand_okuri("あいしあw", &vec!["愛し合"])?;
        assert_eq!(
            got,
            vec!(
                ("あいしあわ".to_string(), vec!("愛し合わ".to_string())),
                ("あいしあうぃ".to_string(), vec!("愛し合うぃ".to_string())),
                ("あいしあうぇ".to_string(), vec!("愛し合うぇ".to_string())),
                ("あいしあを".to_string(), vec!("愛し合を".to_string()))
            ),
        );
        Ok(())
    }
}
