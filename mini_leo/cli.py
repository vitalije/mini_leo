# -*- coding: utf-8 -*-
"""Console script for mini_leo."""
import sys
import click
from mini_leo import test
@click.command()
def main(args=None):
    """Console script for mini_leo."""
    #click.echo("See click documentation at http://click.pocoo.org/")
    if test() != 42:
        return 1
    return 0
if __name__ == "__main__":
    sys.exit(main())  # pragma: no cover
