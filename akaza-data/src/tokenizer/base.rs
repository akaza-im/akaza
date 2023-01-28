pub trait AkazaTokenizer {
    fn tokenize(&self, src: &str) -> anyhow::Result<String>;
}

/// マージ処理に利用する為の中間表現
#[derive(Debug)]
pub(crate) struct IntermediateToken {
    surface: String,
    yomi: String,
    hinshi: String,
    subhinshi: String,
    subsubhinshi: String,
}

impl IntermediateToken {
    pub(crate) fn new(
        surface: String,
        yomi: String,
        hinshi: String,
        subhinshi: String,
        subsubhinshi: String,
    ) -> IntermediateToken {
        IntermediateToken {
            surface,
            yomi,
            hinshi,
            subhinshi,
            subsubhinshi,
        }
    }
}

/// 特定の品詞をマージする
/// ipadic の品詞体系を対象とする。
pub(crate) fn merge_terms_ipadic(intermediates: Vec<IntermediateToken>) -> String {
    let mut buf = String::new();
    let mut i = 0;
    while i < intermediates.len() {
        let token = &intermediates[i];
        let mut surface = token.surface.clone();
        let mut yomi = token.yomi.clone();
        let mut prev_token = token;

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

                助動詞とその前のトークンを単純に接続すると以下の様なケースで困る。

                鈴鹿医療科学技術大学/名詞/固有名詞/すずかいりょうかがくぎじゅつだいがく
                で/助動詞/_/で
                あっ/助動詞/_/あっ
                た/助動詞/_/た
                が/助詞/接続助詞/が
            */
            let token = &intermediates[j];

            if (token.hinshi == "助動詞"
                && (prev_token.hinshi == "動詞" || prev_token.hinshi == "助動詞"))
                || token.subhinshi == "接続助詞"
                || token.subhinshi == "接尾"
            {
                // println!("PREV_TOKEN: {:?}", prev_token);
                // println!("TOKEN: {:?}", token);
                surface += token.surface.as_str();
                yomi += if token.surface == "家"
                    && token.yomi == "か"
                    && prev_token.subsubhinshi == "人名"
                {
                    // 人名 + 家 のケースに ipadic だと「か」と読んでしまう
                    // 問題があるので、その場合は「家/け」に読み替える。
                    "け"
                } else {
                    token.yomi.as_str()
                };

                j += 1;
                prev_token = token;
            } else {
                break;
            }
        }

        buf += format!("{surface}/{yomi} ").as_str();

        i = j;
    }
    buf.trim_end().to_string()
}
