# sn0int [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates] [![Documentation Status][docs-img]][docs]

[travis-img]:   https://travis-ci.org/kpcyrd/sn0int.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/sn0int
[crates-img]:   https://img.shields.io/crates/v/sn0int.svg
[crates]:       https://crates.io/crates/sn0int
[docs-img]:     https://readthedocs.org/projects/sn0int/badge/?version=latest
[docs]:         https://sn0int.readthedocs.io/en/latest/?badge=latest

sn0int is an OSINT framework and package manager. It was built for IT security
professionals and bug hunters to gather intelligence about a given target or
about yourself. sn0int is enumerating attack surface by semi-automatically
processing public information and mapping the results in a unified format for
followup investigations.

Among other things, sn0int is currently able to:

- [X] Harvest subdomains from certificate transparency logs
- [X] Harvest subdomains from various passive dns logs
- [X] Sift through subdomain results for publicly accessible websites
- [X] Harvest emails from pgp keyservers
- [X] Enrich ip addresses with ASN and geoip info
- [X] Harvest subdomains from the wayback machine

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource.  None of the investigations listed above are hardcoded
in the source, instead those are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own since you don't need to send a
pull request.

Join us on IRC: <ircs://irc.hackint.org/#sn0int>

## Installation

- Archlinux: `yaourt -S sn0int`
- Alpine: `apk add --no-cache sqlite-dev libseccomp-dev cargo` + build from source
- Debian: `apt install libsqlite3-dev libseccomp-dev` + build from source
- OpenBSD: `pkg_add sqlite3` + build from source
- OSX: `brew install sqlite3` + build from source

## License

GPLv3+
