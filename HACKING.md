# Make development environment on arch linux

    cd akaza-data/ && pip install -r requirements.txt
    cd ibus-akaza/ && pip install -r requirements.txt
    yay -S kytea python-pytest python-pip cmake marisa python-setuptools

## NOTE

 * for data generation, wikipedia の全データをダウンロードして言語モデルと辞書のロードが行われるために、ディスク容量とメモリと CPU がある程度必要です。


