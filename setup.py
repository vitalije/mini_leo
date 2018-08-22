#!/usr/bin/env python
# -*- coding: utf-8 -*-
#@+leo-ver=5-thin
#@+node:vitalije.20180822183813.1: * @file setup.py
#@@first
#@@first

#@+others
#@+node:vitalije.20180822183827.1: ** Declarations (setup.py)
"""The setup script."""

from setuptools import setup, find_packages

with open('README.md') as readme_file:
    readme = readme_file.read()

with open('HISTORY.md') as history_file:
    history = history_file.read()

requirements = ['Click>=6.0', ]

setup_requirements = ['pytest-runner', ]

test_requirements = ['pytest', ]

setup(
    author="Vitalije Milosevic",
    author_email='vitalije@kviziracija.net',
    classifiers=[
        'Development Status :: 2 - Pre-Alpha',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Natural Language :: English',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
    ],
    description="A minimal version of Leo editor.",
    entry_points={
        'console_scripts': [
            'mini_leo=mini_leo.cli:main',
        ],
    },
    install_requires=requirements,
    license="MIT license",
    long_description=readme + '\n\n' + history,
    include_package_data=True,
    keywords='mini_leo',
    name='mini_leo',
    packages=find_packages(include=['mini_leo']),
    setup_requires=setup_requirements,
    test_suite='tests',
    tests_require=test_requirements,
    url='https://github.com/vitalije/mini_leo',
    version='0.1.0',
    zip_safe=False,
)
#@-others
#@@language python
#@@tabwidth -4
#@-leo
