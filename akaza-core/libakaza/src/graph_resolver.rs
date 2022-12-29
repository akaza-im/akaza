#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_dict::BinaryDict;
    use crate::lm::system_unigram_lm::SystemUnigramLM;
    use daachorse::DoubleArrayAhoCorasick;

    #[test]
    fn test() {
        // let pma = DoubleArrayAhoCorasick::new(patterns).unwrap();

        // let mut it = pma.find_overlapping_iter("abcd");

        let patterns = vec!["わたし", "の", "なまえ", "なかの", "です", "なか"];
        let pma = DoubleArrayAhoCorasick::<usize>::new(patterns);
        let pma = pma.unwrap();
        let target = "わたしのなまえはなかのなのです。";
        let p = pma.find_overlapping_iter(target);
        p.for_each(|f| {
            let p = &target[f.start()..f.end()];
            println!("{} {} {} {}", f.start(), f.end(), f.value(), p);
        });
    }
}
