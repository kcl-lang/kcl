#! /usr/bin/env python
# -*- coding: utf-8 -*-
try:
    from setuptools import setup
except ImportError:
    from distutils.core import setup
import setuptools

"""
1. cd internal
2. python setup.py sdist
3. twine upload dist/*
"""
setup(
    name='kclvm',
    author='KCL Authors',
    version='0.4.2.2',
    license='Apache License 2.0',

    description='KCLVM', 
    long_description='''A constraint-based record & functional language mainly used in configuration and policy scenarios.''',
    author_email='',  
    url='https://kusionstack.io/',  
    packages=setuptools.find_packages(),

    zip_safe=True,
    # 依赖包
    install_requires=[
        'wheel==0.34.2', 
        'twine==3.2.0',
        'pyyaml==5.4',
        'pytest-xdist==2.2.1',
        'lark-parser==0.11.3',
        'filelock==3.6.0',
        'yapf==0.29.0',
        'pytest==6.2.2',
        'pypeg2==2.15.2',
        'protobuf==3.19.4',
        'grequests',
        'schema',
        'coverage',
        'ruamel.yaml',
        'toml',
        'numpydoc',
        'pygls==0.10.3',
        'fastapi',
        'uvicorn',
        'gunicorn==20.1.0',
        'parsy==1.3.0',
        'wasmer==1.0.0',
        'wasmer_compiler_cranelift==1.0.0',
        'pyopenssl'
    ],
)
