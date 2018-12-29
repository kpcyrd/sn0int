# sn0int [![Build Status][travis-img]][travis] [![Crates.io][crates-img]][crates] [![Documentation Status][docs-img]][docs]

[travis-img]:   https://travis-ci.org/kpcyrd/sn0int.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/sn0int
[crates-img]:   https://img.shields.io/crates/v/sn0int.svg
[crates]:       https://crates.io/crates/sn0int
[docs-img]:     https://readthedocs.org/projects/sn0int/badge/?version=latest
[docs]:         https://sn0int.readthedocs.io/en/latest/?badge=latest

sn0int is a semi-automatic OSINT framework and package manager. It was built
for IT security professionals and bug hunters to gather intelligence about a
given target or about yourself. sn0int is enumerating attack surface by
semi-automatically processing public information and mapping the results in a
unified format for followup investigations.

Among other things, sn0int is currently able to:

- [X] Harvest subdomains from certificate transparency logs
- [X] Harvest subdomains from various passive dns logs
- [X] Sift through subdomain results for publicly accessible websites
- [X] Harvest emails from pgp keyservers
- [X] Enrich ip addresses with ASN and geoip info
- [X] Harvest subdomains from the wayback machine
- [X] Gather information about phonenumbers
- [X] Bruteforce interesting urls

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource.  None of the investigations listed above are hardcoded
in the source, instead those are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own since you don't need to send a
pull request.

Join us on IRC: <ircs://irc.hackint.org/#sn0int>

[![asciicast](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB.svg)](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB)

## Getting started

- [Installation](https://sn0int.readthedocs.io/en/latest/install.html)
  - [Archlinux](https://sn0int.readthedocs.io/en/latest/install.html#archlinux)
  - [Debian/Ubuntu/Kali](https://sn0int.readthedocs.io/en/latest/install.html#debian-ubuntu-kali)
  - [Alpine](https://sn0int.readthedocs.io/en/latest/install.html#alpine)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/install.html#openbsd)
  - [Mac OSX](https://sn0int.readthedocs.io/en/latest/install.html#mac-osx)
  - [Windows](https://sn0int.readthedocs.io/en/latest/install.html#windows)
- [Running your first investigation](https://sn0int.readthedocs.io/en/latest/usage.html)
  - [Installing the default modules](https://sn0int.readthedocs.io/en/latest/usage.html#installing-the-default-modules)
  - [Adding something to scope](https://sn0int.readthedocs.io/en/latest/usage.html#adding-something-to-scope)
  - [Running a module](https://sn0int.readthedocs.io/en/latest/usage.html#running-a-module)
  - [Running followup modules on the results](https://sn0int.readthedocs.io/en/latest/usage.html#running-followup-modules-on-the-results)
  - [Unscoping entities](https://sn0int.readthedocs.io/en/latest/usage.html#unscoping-entities)
- [Scripting](https://sn0int.readthedocs.io/en/latest/scripting.html)
  - [Write your first module](https://sn0int.readthedocs.io/en/latest/scripting.html#write-your-first-module)
  - [Publish your module](https://sn0int.readthedocs.io/en/latest/scripting.html#publish-your-module)
- [Database](https://sn0int.readthedocs.io/en/latest/database.html)
  - [db_add](https://sn0int.readthedocs.io/en/latest/database.html#db-add)
  - [db_update](https://sn0int.readthedocs.io/en/latest/database.html#db-update)
  - [db_select](https://sn0int.readthedocs.io/en/latest/database.html#db-select)
- [Keyring](https://sn0int.readthedocs.io/en/latest/keyring.html)
  - [Managing the keyring](https://sn0int.readthedocs.io/en/latest/keyring.html#managing-the-keyring)
  - [Using access keys in scripts](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-in-scripts)
  - [Using access keys as source argument](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-as-source-argument)
- [Function reference](https://sn0int.readthedocs.io/en/latest/reference.html)

## License

GPLv3+
