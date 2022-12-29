use crate::kana_trie::KanaTrie;

/**
 * ユーザー固有データ
 */
struct UserData {
    /// 読み仮名のトライ。入力変換時に共通接頭辞検索するために使用。
    kana_trie: KanaTrie,
}
