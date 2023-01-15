use std::collections::HashMap;

pub fn merge_skkdict(dicts: Vec<HashMap<String, Vec<String>>>) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for dict in dicts {
        for (yomi, kanjis) in dict {
            let target = result.entry(yomi).or_default();
            for kanji in kanjis {
                target.push(kanji);
            }
        }
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
