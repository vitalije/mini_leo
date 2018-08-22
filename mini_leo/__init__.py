# -*- coding: utf-8 -*-
"""Top-level package for Mini Leo."""
from mini_leo._native import ffi, lib
def test():
    return lib.a_function_from_rust()

__author__ = """Vitalije Milosevic"""
__email__ = 'vitalije@kviziracija.net'
__version__ = '0.1.0'
