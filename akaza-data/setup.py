import os

from setuptools import setup

os.system("make binary")
# extsuffix = subprocess.run(["python3-config", "--extension-suffix"], capture_output=True).stdout

setup(
    name="akaza-data",
    version="0.0.1",
    install_requires=["marisa-trie==0.7.5", "jaconv==0.2.4"],
    packages=['akaza_data', 'akaza_data.data'],
    package_data={
        'akaza_data': ['*.so'],
        'akaza_data.data': ['system_*.trie', 'emoji.trie', 'lm_*.trie']
    },
    extras_require={
    },
    entry_points={
    }
)
