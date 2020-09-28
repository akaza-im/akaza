import datetime
import pathlib
import shutil
import subprocess


def get_sig():
    hash = subprocess.run(["git", "rev-parse", "--short", 'HEAD'], capture_output=True).stdout.decode(
        'utf-8').rstrip()
    sig = datetime.datetime.now().strftime('%Y%m%d-%H%M') + "-" + hash
    return sig


def mkdir_p(path: str):
    pathlib.Path(path).mkdir(exist_ok=True, parents=True)


def copy_snapshot(path: str):
    sig = get_sig()
    name = pathlib.Path(path).name
    pathlib.Path('work/dump').mkdir(exist_ok=True, parents=True)
    shutil.copy(path, f'work/dump/{sig}-{name}')
