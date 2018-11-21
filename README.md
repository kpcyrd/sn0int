# sn0int [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates] [![Documentation Status][docs-img]][docs]

[travis-img]:   https://travis-ci.org/kpcyrd/sn0int.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/sn0int
[crates-img]:   https://img.shields.io/crates/v/sn0int.svg
[crates]:       https://crates.io/crates/sn0int
[docs-img]: https://readthedocs.org/projects/sn0int/badge/?version=latest
[docs]: https://sn0int.readthedocs.io/en/latest/?badge=latest

sn0int is an OSINT framework and package manager. Its purpose is
semi-automatically processing public information to enumerate attack surface.
sn0int itself is only providing an engine that can be extended with scripts.

sn0int is heavily inspired by recon-ng, but takes a few different design
approaches. We've tried to correct some limitations in the database design and
also addressed the modularity problem:

Instead of downloading and reviewing python scripts that have full access to
your system, sn0int is executing modules in a lua sandbox. Modules can be
published to the sn0int registry and then installed by users. This means that
you don't have to send pull requests to sn0int to add a module and updates can
be shipped much faster.

Join us on IRC: <ircs://irc.hackint.org/#sn0int>

## Installation

- Archlinux: `yaourt -S sn0int`
- Alpine: `apk add --no-cache sqlite-dev libseccomp-dev cargo` + build from source
- Debian: `apt install libsqlite3-dev libseccomp-dev` + build from source
- OpenBSD: `pkg_add sqlite3` + build from source
- OSX: `brew install sqlite3` + build from source

## License

GPLv3+
