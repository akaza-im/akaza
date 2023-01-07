use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Result;

use crate::graph::word_node::WordNode;

/// フルアノテーションコーパス
/// Kytea のフルアノテーションコーパスと同様の形式です。
///
/// コーパス/こーぱす の/の 文/ぶん で/で す/す 。/。
///
/// ↑のような形式です。
///
/// 品詞を取り扱うのは素人には難しいので、品詞というよりは、
/// どの位置で変換候補が区切られていたら気持ち良いか、という
/// 観点で区切る想定です。
///
/// http://www.phontron.com/kytea/io-ja.html
pub struct FullAnnotationCorpus {
    pub nodes: Vec<WordNode>,
}

impl FullAnnotationCorpus {
    /// フルアノテーションコーパスをパースする。
    pub fn new(src: &str) -> FullAnnotationCorpus {
        let p: Vec<&str> = src.split(' ').collect();
        let mut start_pos = 0;
        let mut nodes: Vec<WordNode> = Vec::new();
        for x in p {
            let (surface, yomi) = x.split_once('/').unwrap();
            nodes.push(WordNode::new(start_pos, surface, yomi));
            start_pos += yomi.len() as i32;
        }
        FullAnnotationCorpus { nodes }
    }

    /// コーパスの「よみ」を連結したものを返す。
    pub fn yomi(&self) -> String {
        let mut buf = String::new();
        for yomi in self.nodes.iter().map(|f| f.yomi.as_str()) {
            buf += yomi;
        }
        buf
    }

    /// 正解ノードオブジェクトのセットを返す
    pub fn correct_node_set(&self) -> HashSet<WordNode> {
        HashSet::from_iter(self.nodes.iter().cloned())
    }
}

pub fn read_corpus_file(src: &Path) -> Result<Vec<FullAnnotationCorpus>> {
    let mut result = Vec::new();

    let file = File::open(src)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.starts_with(";;") {
            // コメント行はスキップ。
            continue;
        }
        if line.trim().is_empty() {
            // 空行はスキップ
            continue;
        }
        result.push(FullAnnotationCorpus::new(line.trim()));
    }
    Ok(result)
}
