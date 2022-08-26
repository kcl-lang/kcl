#! /usr/bin/env python
# -*- coding: utf-8 -*-
import pathlib
try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup
from pkg_resources import require
import setuptools

# Steps:
# 1. cd internal
# 2. modify kclvm version in setup.py
# 3. run `python setup.py sdist` to build package
# 4. run `twine upload dist/kclvm-0.x.x` to upload package to PyPI
# 5. input username and password of PyPI

install_requires = []
require_path = pathlib.Path(__file__).parent.joinpath("kclvm/scripts/requirements.txt")
with open(require_path) as f:
    requires = f.read().split('\n')
    for require in requires:
        install_requires.append(require)

setup(
    name='kclvm',
    author='KCL Authors',
    version='0.4.3',
    license='Apache License 2.0',

    description='KCLVM', 
    long_description='''A constraint-based record & functional language mainly used in configuration and policy scenarios.''',
    author_email='',  
    url='https://kusionstack.io/',  
    packages=setuptools.find_packages(),
    include_package_data=True,
    zip_safe=True,
    # 依赖包
    install_requires=install_requires,
)
