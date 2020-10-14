import os
from setuptools import setup

os.system("make binary")

setup(
    name="pyakaza",
    version="0.0.2",
    install_requires=[],
    packages=['pyakaza'],
    package_data={
        'akaza_data': ['*.so'],
    },
    extras_require={
    },
    entry_points={
    }
)
