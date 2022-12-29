#[cfg(test)]
mod tests {
    use crate::binary_dict::KanaKanjiDict;
    use daachorse::DoubleArrayAhoCorasick;
    use std::collections::HashSet;

    #[test]
    fn test() {
        // let pma = DoubleArrayAhoCorasick::new(patterns).unwrap();

        // let mut it = pma.find_overlapping_iter("abcd");

        let dict = KanaKanjiDict::load(
            &"/home/tokuhirom/dev/akaza/akaza-data/data/system_dict.trie".to_string(),
        )
        .expect("KanaKanjiDict can open");
        let yomis = dict.all_yomis();
        println!("Make unique");
        let yomis: HashSet<String> = yomis.into_iter().collect();
        let yomicnt = yomis.len();
        assert_eq!(yomicnt, 3);

        // let patterns = vec!["わたし", "の", "なまえ", "なかの", "です", "なか"];
        let patterns = yomis;
        println!("Build it. {}", yomicnt);
        let pma = DoubleArrayAhoCorasick::<usize>::new(patterns);
        println!("Build it.");
        let pma = pma.unwrap();
        let target = "わたしのなまえはなかのなのです。";
        let p = pma.find_overlapping_iter(target);
        p.for_each(|f| {
            let p = &target[f.start()..f.end()];
            println!("{} {} {} {}", f.start(), f.end(), f.value(), p);
        });
    }
}
