from setuptools import setup

setup(
    name="akaza",
    version="0.0.1",
    install_requires=["marisa-trie==0.7.5", "jaconv==0.2.4"],
    extras_require={
        "develop": ["dev-packageA", "dev-packageB"]
    },
    entry_points={
    }
)
