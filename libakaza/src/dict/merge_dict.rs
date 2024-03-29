use std::collections::HashMap;

pub fn merge_dict(dicts: Vec<HashMap<String, Vec<String>>>) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for dict in dicts {
        for (yomi, kanjis) in dict {
            let target = result.entry(yomi).or_default();
            for kanji in kanjis {
                if !target.contains(&kanji) {
                    target.push(kanji);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_dict() {
        let got = merge_dict(vec![
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

    #[test]
    fn test_merge_dict_dedup() {
        let got = merge_dict(vec![
            HashMap::from([("ご".to_string(), vec!["語".to_string()])]),
            HashMap::from([("ご".to_string(), vec!["語".to_string(), "碁".to_string()])]),
        ]);
        assert_eq!(
            got,
            HashMap::from([("ご".to_string(), vec!["語".to_string(), "碁".to_string()]),])
        );
    }
}
