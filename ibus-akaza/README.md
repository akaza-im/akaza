# ibus-akaza

akaza の ibus binding です。

開発状態: 一応動きますが、まだ未実装の機能があります。

## install

`make && sudo make install` してください。

## Debugging

`sudo make install-debug` とすると、`../target/debug/ibus-akaza` を利用して起動するように `/usr/share/ibus/component/akaza.xml` が設定されます。

この状態で `./ibus-akaza-debug-session.sh` を実行すると、ibus が再起動されて、ログファイルが表示されるようになります。
