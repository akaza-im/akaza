import sys
import glob
import subprocess

def split(a, n):
    k, m = divmod(len(a), n)
    return (a[i * k + min(i, m):(i + 1) * k + min(i + 1, m)] for i in range(n))

files = glob.glob('text/*/wiki_*')
chunks = split(files, 7)

procs = []
for chunk in chunks:
    proc = subprocess.Popen(['python', 'bin/wiki2text.py'] + chunk)
    procs.append(proc)

for proc in procs:
    ret = proc.wait()
    print(ret)

