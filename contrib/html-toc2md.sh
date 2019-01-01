#!/bin/sh
perl -n -e '/toctree-l(\d).*href="([^"]+)">(.+)<\/a/ && print $1==2?"  ":"", "- [$3](https://sn0int.readthedocs.io/en/latest/$2)\n"' < docs/_build/html/index.html
