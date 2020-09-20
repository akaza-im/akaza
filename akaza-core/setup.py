from setuptools import setup

setup(
    name="akaza",
    version="0.0.2",
    install_requires=["marisa-trie==0.7.5", "jaconv==0.2.4"],
    packages=['akaza', 'akaza.tinylisp'],
    extras_require={
    },
    entry_points={
    }
)
