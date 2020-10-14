import os
from setuptools import setup

os.system("make binary")

setup(
    name="pyakaza",
    version="0.0.3",
    install_requires=[],
    packages=['pyakaza'],
    package_data={
        'pyakaza': ['*.so'],
    },
    extras_require={
    },
    entry_points={
    },
    zip_safe=False
)
