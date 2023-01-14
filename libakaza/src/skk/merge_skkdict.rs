use std::collections::{HashMap, HashSet};

pub fn merge_skkdict(dicts: Vec<HashMap<String, Vec<String>>>) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    // 取りうるキーをリストアップする
    let mut keys: HashSet<String> = HashSet::new();
    for dic in &dicts {
        for key in dic.keys() {
            keys.insert(key.to_string());
        }
    }

    // それぞれのキーについて、候補をリストアップする
    for key in keys {
        let mut kanjis: Vec<String> = Vec::new();

        for dic in &dicts {
            if let Some(kkk) = dic.get(&key.to_string()) {
                for k in kkk {
                    if !kanjis.contains(k) {
                        kanjis.push(k.clone());
                    }
                }
            }
        }

        result.insert(key, kanjis);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_skkdict() {
        let got = merge_skkdict(vec![
            HashMap::from([("ご".to_string(), vec!["語".to_string()])]),
            HashMap::from([
                ("ご".to_string(), vec!["後".to_string(), "碁".to_string()]),
                ("お".to_string(), vec!["緒".to_string()]),
            ]),
        ]);
        assert_eq!(
            got,
            HashMap::from([
                (
                    "ご".to_string(),
                    vec!["語".to_string(), "後".to_string(), "碁".to_string()]
                ),
                ("お".to_string(), vec!["緒".to_string()])
            ])
        );
    }
}
