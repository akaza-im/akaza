from setuptools import setup

setup(
    name="akaza-data",
    version="0.0.1",
    install_requires=["marisa-trie==0.7.5", "jaconv==0.2.4"],
    packages=['akaza_data', 'akaza_data.data'],
    package_data={'akaza_data.data': ['system_*.trie', 'emoji.trie']},
    extras_require={
    },
    entry_points={
    }
)
