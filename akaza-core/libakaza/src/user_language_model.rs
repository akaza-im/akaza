use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(PartialEq)]
enum GramType {
    BiGram,
    UniGram,
}

// unigram 用と bigram 用のロジックでコピペが増えがちで危ない。
// 完全に分離したほうが良い。
pub struct UserLanguageModel {
    unigram_path: String,
    bigram_path: String,

    need_save: bool,

    // unigram 言語モデルに登録されている、読み仮名を登録しておく。
    // これにより、「ひょいー」などの漢字ではないものを、単語として IME が認識できるように
    // している。
    // 本質的には、user language model でやるべき処理というよりも、ユーザー単語辞書でもつく
    // ってやるのが本筋だと思わなくもない
    unigram_kanas: HashSet<String>,

    /// ユニーク単語数
    unigram_c: u32,
    /// 総単語出現数
    unigram_v: u32,
    unigram: HashMap<String, u32>,

    bigram_c: u32,
    bigram_v: u32,
    bigram: HashMap<String, u32>,

    alpha: f32, // = 0.00001;
}

impl UserLanguageModel {
    fn new(unigram_path: &String, bigram_path: &String) -> UserLanguageModel {
        UserLanguageModel {
            unigram_path: unigram_path.clone(),
            bigram_path: bigram_path.clone(),
            need_save: false,
            unigram_kanas: HashSet::new(),
            unigram_c: 0,
            unigram_v: 0,
            unigram: HashMap::new(),
            bigram_c: 0,
            bigram_v: 0,
            bigram: HashMap::new(),
            alpha: 0.00001,
        }
    }

    fn read(
        &mut self,
        path: &String,
        gram_type: GramType,
    ) -> Result<(u32, u32, HashMap<String, u32>), String> {
        let mut c = 0;
        let mut v = 0;
        let mut map = HashMap::new();

        // TODO : 厳密なエラー処理
        let Ok(file) = File::open(path) else {
            return Err("Cannot open user language model file".to_string());
        };

        for line in BufReader::new(file).lines() {
            let Ok(line) = line else {
                return Err("Cannot read user language model file".to_string());
            };
            let tokens: Vec<&str> = line.trim().splitn(2, " ").collect();
            if tokens.len() != 2 {
                continue;
            }

            let key = tokens[0];
            let Ok(count) = tokens[1].to_string().parse::<u32>() else {
                return Err("Invalid line in user language model: ".to_string() + tokens[1]);
            };

            map.insert(key.to_string(), count);

            // unigram 言語モデルに登録されている、ひらがなの単語を、集めて登録しておく。
            // これにより、「ひょいー」などの漢字ではないものを、単語として IME が認識できるように
            // している。
            // 本質的には、user language model でやるべき処理というよりも、ユーザー単語辞書でもつく
            // ってやるのが本筋だと思わなくもない
            if gram_type == GramType::UniGram {
                let tokens: Vec<&str> = line.splitn(2, "/").collect();
                if tokens.len() != 2 {
                    continue;
                }
                let kana = tokens[0];
                self.unigram_kanas.insert(kana.to_string());
            }

            c += count;
            v += 1;
        }

        return Ok((c, v, map));
    }

    fn load_unigram(&mut self) -> Result<(), String> {
        let result = self.read(&self.unigram_path.clone(), GramType::UniGram);
        let Ok((c, v, map)) = result else {
            return Err(result.err().unwrap());
        };
        self.unigram_c = c;
        self.unigram_v = v;
        self.unigram = map;
        return Ok(());
    }

    fn load_bigram(&mut self) -> Result<(), String> {
        let result = self.read(&self.bigram_path.clone(), GramType::BiGram);
        let Ok((c, v, map)) = result else {
            return Err(result.err().unwrap());
        };
        self.bigram_c = c;
        self.bigram_v = v;
        self.bigram = map;
        return Ok(());
    }

    fn add_entry(&mut self, nodes: &Vec<crate::node::Node>) {
        // unigram
        for node in nodes {
            let key = &node.key;
            if !self.unigram.contains_key(key) {
                // increment unique count
                self.unigram_c += 1;
            }
            self.unigram_v += 1;
            let tokens: Vec<&str> = key.splitn(2, "/").collect();
            let kana = tokens[1];
            // std::wstring kana = std::get<1>(split2(key, L'/', splitted));
            self.unigram_kanas.insert(kana.to_string());
            self.unigram.insert(
                key.to_string(),
                self.unigram.get(key.as_str()).unwrap_or(&0_u32) + 1,
            );
        }

        // bigram
        for i in 1..nodes.len() {
            let Some(node1) = nodes.get(i - 1) else {
                continue;
            };
            let Some(node2) = nodes.get(i) else {
                continue;
            };

            let k1 = &node1.key.to_string().clone();
            let key = k1.to_string() + &"\t".to_string() + &node2.key;
            if self.bigram.contains_key(&key) {
                self.bigram_c += 1;
            }
            self.bigram_v += 1;
            self.bigram
                .insert(key.clone(), self.bigram.get(&key).unwrap_or(&0) + 1);
        }

        self.need_save = true;
    }

    pub(crate) fn get_unigram_cost(&self, key: &String) -> Option<f32> {
        let Some(count) = self.unigram.get(key) else {
            return None;
        };
        return Some(f32::log10(
            ((*count as f32) + self.alpha)
                / ((self.unigram_c as f32) + self.alpha + (self.unigram_v as f32)),
        ));
    }

    fn get_bigram_cost(&self, key1: &String, key2: &String) -> Option<f32> {
        let key = key1.clone() + "\t" + key2;
        let Some(count) = self.bigram.get(key.as_str()) else {
            return None;
        };
        return Some(f32::log10(
            ((*count as f32) + self.alpha)
                / ((self.bigram_c as f32) + self.alpha + (self.bigram_v as f32)),
        ));
    }

    // TODO save_file

    /*

    void akaza::UserLanguageModel::save_file(const std::string &path, const std::unordered_map<std::wstring, int> &map) {
        std::string tmppath(path + ".tmp");
        std::wofstream ofs(tmppath, std::ofstream::out);
        ofs.imbue(std::locale(std::locale(), new std::codecvt_utf8<wchar_t>));

        for (const auto&[words, count] : map) {
            ofs << words << " " << count << std::endl;
        }
        ofs.close();

        int status = std::rename(tmppath.c_str(), path.c_str());
        if (status != 0) {
            std::string err = strerror(errno);
            throw std::runtime_error(err + " : " + path + " (Cannot write user language model)");
        }
    }
    */
}
