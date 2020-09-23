import glob
import subprocess
from subprocess import TimeoutExpired
import logging
import multiprocessing
import time


def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))


logging.basicConfig(level=logging.DEBUG)

numcpu = multiprocessing.cpu_count()
logging.info(f"numcpu={numcpu}")

files = glob.glob('work/extracted/*/wiki_*')
chunks = split(files, numcpu)

t0 = time.time()

finished = 0

procs = []
for chunk in chunks:
    cmd = ['python', 'bin/011_wiki2text.py'] + chunk
    proc = subprocess.Popen(cmd)
    procs.append(proc)

    # logging.info(f"Run {cmd}")

    while numcpu <= len(procs):
        for proc in procs:
            try:
                logging.info(f"Waiting {proc}")
                retcode = proc.wait(timeout=1)
                finished += 1
                logging.info(
                    f"retcode={retcode} progress={finished}/{len(files)} (Elapsed: {time.time() - t0}, Expected: {(time.time() - t0) * len(files) / finished}, current procs={len(procs)})")
                procs.remove(proc)
                break
            except TimeoutExpired:
                logging.info(f"Timeout: {proc}")

for proc in procs:
    retcode = proc.wait()
    logging.info(f"retcode={retcode}")
