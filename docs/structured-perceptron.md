# 構造化パーセプトロン

https://tech.preferred.jp/ja/blog/nlp2011-inputmethod-structuredsvm/

統計的かな漢字変換ではなく、識別モデルを利用するパターンも試してみたい。

https://www.anlp.jp/proceedings/annual_meeting/2011/pdf_dir/C4-3.pdf

Mozc で統計的かな漢字変換を採用しているのは以下のようなところもあるのかなと。
Google の収集した Web のデータを使えるというのがやはり Mozc の強みの一つなので、それを活かせる環境ならば、統計的かな漢字変換がとても有利なのだと思う。

> 識別学習のモデル構築時間を生成モデル並にするには, 学習用の
> コーパスを小さくするしかなく, これでは Web の 統計を反映しにくくなる.

一方、Akaza では潤沢な計算資源も使えず、ウェブコーパスもできる限り避けたいな､と思っているので、そうなってくると使えるコーパスはごく少量になってくる。
となると、案外、識別モデルのほうがいいんじゃないかなぁ、と。

統計的かな漢字変換で、スコアをチューニングしていくということになると、パラメータを職人が調整する、みたいな感じになってくるので、それは OSS の世界だと成立が
難しいと思う。
こっちのパラメータいじったらこっちの変換結果がぶっ壊れた､みたいなことになりがちだと思うので。

識別モデルの場合、限られた少数のデータをもとに、そのコーパスにフィットした形にチューニングするということができる。はず。

正直、Wikipedia のコーパスだけだと、代表性が微妙だなぁと思うので Wikipedia をベースに学習させて、その上で誤変換される例をもとに識別モデルでチューニングしていく、という方針がよいのではないかと。

## 識別モデルの実装方針

日本語入力本でも、まずは構造化パーセプトロンを実装し、その上で構造化SVMなど他のメソッドを利用してチューニングしていくのが良い、と書いてあるので、
それにそってすすめていきたい。

構造化パーセプトロンでは、ビタビアルゴリズムで文を生成して、それが教師データと一致していればなにもしない。
一致していない場合には、正解するノードのコストを1上げる。不正解ノードを1下げるという処理を行う。

という非常にシンプルな方法で実現できるので、頑張って実装してみてもいいのかなぁ、と。

正直、自分自身が品詞やらなんやらの知識がないのもあるし、品詞がどうこうとかいうと論争のもとっぽさを感じるので、、
そういう意味でも、教師データを用意すりゃ精度が上がりますよ~。誤変換が気になるようなら教師データ足してくださいねー。
というのはちょうどよい温度感なのかなーと思う。

また、個人的には基本のデータセットは、公開されたデータセットをベースにするのが良いと思っているんだけど。

## 学習の進みが遅いケース

基本的に、コストがノード単位に振られているから、長い文節がマッチし易い傾向にある。
これはこれで良いことなのだが、

> 洗濯物/せんたくもの を/を 干す/ほす の/の が/が 面倒/めんどう だ/だ

のようなケースの場合、"逃が/のが" が辞書に登録されていると、 "の/の が/が" の2文節を通るよりもコストがやすくなりがち。
なので、めちゃくちゃコストが下がっていくのをまたないといけない感じになりがち。

このへんは、未知語のコストを統計的かな漢字変換のときに `20` とか雑にデカくつけてるのがよくない。
しかもここがハードコードされている。こういうのを調整可能にしないといけない。

こういう、統計的かな漢字変換前提でハードコードされている部分とかをばらしていかないといけない。
クラス構造とかデータの持ち方を調整しないと、ごちゃごちゃしすぎているので、調整が必要。

例えば、「よくない」の変換結果として「翼内」が一位にくるけど「良くない」がトップに出てくるべきかもしれない。
そういった調整は、識別モデルのほうがしやすい、のかな?

ユーザーの入力結果の学習についても、統一的に扱えるような気がする。
