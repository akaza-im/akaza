- anthy
  - http://www.fenix.ne.jp/~G-HAL/soft/nosettle/anthy.html#patch13
  - http://vagus.seesaa.net/article/62333234.html
- google cgi api for japanese input
  - https://www.google.co.jp/ime/cgiapi.html

## Data storage library

 * かな漢字変換においては、共通接頭辞検索ができるライブラリが必要。
   * Trie(エッジに文字情報がついたツリー)構造がよく使われる。
     * Double Array や LOUDS などがある
     * LOUDS はサイズが小さくなるが、動的な追加削除はできない。
     * 検索速度は Double Array のほうが速い

 * comb には何に trie を作成しているのか?
   * ユーザー辞書とシステム辞書とシステム言語モデルに利用している。
   * ただし、システム言語モデルは、純粋なキーからの検索しかしていない。

 * 本ライブラリのゴールは以下の通り。
   * メンテナンスコストが低いこと
   * 異常に遅かったり以上に時間がかかったりしないこと。
   * Disk 上で mmap してアクセス可能だとよい。

 * [marisa-trie](http://www.s-yata.jp/marisa-trie/docs/readme.en.html)
   * The dictionary format of libmarisa **depends on the architecture**.
   * LOUDS nested patricia trie
   * Has disk based support
 * [libdatrie](https://github.com/tlwg/libdatrie)
   * The trie data is portable across platforms. The byte order in the disk is always little-endian, and is
     read correctly on either little-endian or big-endian systems.
 * [DAWG](https://code.google.com/archive/p/dawgdic/)
   * [python binding](https://github.com/pytries/DAWG)
   * C++ library last update was 2012.
 * [Darts](http://chasen.org/~taku/software/darts/)
   * Maybe, not portable.
   * Last release: 2008
 * [cedar](http://www.tkl.iis.u-tokyo.ac.jp/~ynaga/cedar/)
   * Last release: 2014
 * [dastrie](http://www.chokkan.org/software/dastrie/)
   * Last update: 2008
 * [hat-trie](https://github.com/dcjones/hat-trie)
   * No disk based support
