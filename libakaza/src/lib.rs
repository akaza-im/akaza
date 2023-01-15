#![allow(dead_code)]

extern crate core;

pub mod config;
pub mod corpus;
pub mod cost;
pub mod dict;
pub mod engine;
pub mod extend_clause;
pub mod graph;
pub mod kana;
pub mod kana_trie;
pub mod lm;
pub mod romkan;
pub mod skk;
mod tinylisp;
pub mod trie;
pub mod user_side_data;

// ↓ TODO: これは不要
const UNKNOWN_WORD_ID: i32 = -1;
