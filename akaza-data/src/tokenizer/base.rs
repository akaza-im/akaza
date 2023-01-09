use std::collections::HashSet;

pub trait AkazaTokenizer {
    fn tokenize(&self, src: &str) -> anyhow::Result<String>;
}

/// マージ処理に利用する為の中間表現
pub(crate) struct IntermediateToken {
    surface: String,
    yomi: String,
    hinshi: String,
    subhinshi: String,
}
impl IntermediateToken {
    pub(crate) fn new(
        surface: String,
        yomi: String,
        hinshi: String,
        subhinshi: String,
    ) -> IntermediateToken {
        IntermediateToken {
            surface,
            yomi,
            hinshi,
            subhinshi,
        }
    }
}

/// 特定の品詞をマージする
pub(crate) fn merge_terms(
    intermediates: Vec<IntermediateToken>,
    mergeable_hinshi: &HashSet<&str>,
    mergeable_subhinshi: &HashSet<&str>,
) -> String {
    let mut buf = String::new();
    let mut i = 0;
    while i < intermediates.len() {
        let token = &intermediates[i];
        let mut surface = token.surface.clone();
        let mut yomi = token.yomi.clone();

        let mut j = i + 1;
        while j < intermediates.len() {
            /*
               実施/名詞/サ変接続/じっし
               さ/動詞/自立/さ
               れ/動詞/接尾/れ
               た/助動詞/_/た

               のような場合、"実施,された"に連結したい。

                書い/動詞/自立/かい
                て/助詞/接続助詞/て
                い/動詞/非自立/い
                た/助動詞/_/た
                もの/名詞/非自立/もの
                で/助動詞/_/で
                ある/助動詞/_/ある

                を、"書いて、いた、ものである" ぐらいまで連結する。
            */
            let token = &intermediates[j];
            if mergeable_hinshi.contains(token.hinshi.as_str())
                || mergeable_subhinshi.contains(token.subhinshi.as_str())
            {
                surface += token.surface.as_str();
                yomi += token.yomi.as_str();
                j += 1;
            } else {
                break;
            }
        }

        buf += format!("{}/{} ", surface, yomi).as_str();

        i = j;
    }
    buf.trim_end().to_string()
}
