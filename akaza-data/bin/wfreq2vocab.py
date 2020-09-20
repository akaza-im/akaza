import logging

with open('work/jawiki.wfreq', 'r') as rfp, \
        open('work/jawiki.vocab', 'w') as wfp:
    vocab = []
    for line in rfp:
        m = line.rstrip().split(' ')
        if len(m) == 2:
            word, cnt = m
            if word.endswith('/UNK'):
                logging.info(f"Skip: {word}(unknown word)")
            elif '/' not in word:
                logging.info(f"Skip: {word}(no slash)")
            elif int(cnt) > 15 and len(word) > 0:
                vocab.append(word)
            else:
                logging.info(f"Skip: {word}: {cnt}(few count)")

    for word in sorted(vocab):
        wfp.write(word + "\n")
