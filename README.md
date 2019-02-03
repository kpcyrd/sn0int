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

- Harvest subdomains from certificate transparency logs
- Harvest subdomains from various passive dns logs
- Sift through subdomain results for publicly accessible websites
- Harvest emails from pgp keyservers
- Enrich ip addresses with ASN and geoip info
- Harvest subdomains from the wayback machine
- Gather information about phonenumbers
- Bruteforce interesting urls

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource.  None of the investigations listed above are hardcoded
in the source, instead those are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own since you don't need to send a
pull request.

Join us on IRC: [irc.hackint.org:6697/#sn0int](https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int)

[![asciicast](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB.svg)](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB)

## Getting started

- [Installation](https://sn0int.readthedocs.io/en/latest/install.html)
  - [Archlinux](https://sn0int.readthedocs.io/en/latest/install.html#archlinux)
  - [Mac OSX](https://sn0int.readthedocs.io/en/latest/install.html#mac-osx)
  - [Debian testing/Debian sid/Kali](https://sn0int.readthedocs.io/en/latest/install.html#debian-testing-debian-sid-kali)
  - [Ubuntu/Debian stable](https://sn0int.readthedocs.io/en/latest/install.html#ubuntu-debian-stable)
  - [Docker](https://sn0int.readthedocs.io/en/latest/install.html#docker)
  - [Alpine](https://sn0int.readthedocs.io/en/latest/install.html#alpine)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/install.html#openbsd)
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
  - [Reading data from stdin](https://sn0int.readthedocs.io/en/latest/scripting.html#reading-data-from-stdin)
- [Database](https://sn0int.readthedocs.io/en/latest/database.html)
  - [db_add](https://sn0int.readthedocs.io/en/latest/database.html#db-add)
  - [db_update](https://sn0int.readthedocs.io/en/latest/database.html#db-update)
  - [db_select](https://sn0int.readthedocs.io/en/latest/database.html#db-select)
- [Keyring](https://sn0int.readthedocs.io/en/latest/keyring.html)
  - [Managing the keyring](https://sn0int.readthedocs.io/en/latest/keyring.html#managing-the-keyring)
  - [Using access keys in scripts](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-in-scripts)
  - [Using access keys as source argument](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-as-source-argument)
- [Configuration](https://sn0int.readthedocs.io/en/latest/config.html)
  - [Configuring a proxy](https://sn0int.readthedocs.io/en/latest/config.html#configuring-a-proxy)
- [Sandbox](https://sn0int.readthedocs.io/en/latest/sandbox.html)
  - [Linux](https://sn0int.readthedocs.io/en/latest/sandbox.html#linux)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/sandbox.html#openbsd)
  - [IPC Protocol](https://sn0int.readthedocs.io/en/latest/sandbox.html#ipc-protocol)
  - [Limitations](https://sn0int.readthedocs.io/en/latest/sandbox.html#limitations)
  - [Diagnosing a sandbox failure](https://sn0int.readthedocs.io/en/latest/sandbox.html#diagnosing-a-sandbox-failure)
- [Function reference](https://sn0int.readthedocs.io/en/latest/reference.html)
  - [clear_err](https://sn0int.readthedocs.io/en/latest/reference.html#clear-err)
  - [datetime](https://sn0int.readthedocs.io/en/latest/reference.html#datetime)
  - [db_add](https://sn0int.readthedocs.io/en/latest/reference.html#db-add)
  - [db_add_ttl](https://sn0int.readthedocs.io/en/latest/reference.html#db-add-ttl)
  - [db_select](https://sn0int.readthedocs.io/en/latest/reference.html#db-select)
  - [db_update](https://sn0int.readthedocs.io/en/latest/reference.html#db-update)
  - [dns](https://sn0int.readthedocs.io/en/latest/reference.html#dns)
  - [error](https://sn0int.readthedocs.io/en/latest/reference.html#error)
  - [asn_lookup](https://sn0int.readthedocs.io/en/latest/reference.html#asn-lookup)
  - [geoip_lookup](https://sn0int.readthedocs.io/en/latest/reference.html#geoip-lookup)
  - [html_select](https://sn0int.readthedocs.io/en/latest/reference.html#html-select)
  - [html_select_list](https://sn0int.readthedocs.io/en/latest/reference.html#html-select-list)
  - [http_mksession](https://sn0int.readthedocs.io/en/latest/reference.html#http-mksession)
  - [http_request](https://sn0int.readthedocs.io/en/latest/reference.html#http-request)
  - [http_send](https://sn0int.readthedocs.io/en/latest/reference.html#http-send)
  - [info](https://sn0int.readthedocs.io/en/latest/reference.html#info)
  - [json_decode](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode)
  - [json_decode_stream](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode-stream)
  - [json_encode](https://sn0int.readthedocs.io/en/latest/reference.html#json-encode)
  - [keyring](https://sn0int.readthedocs.io/en/latest/reference.html#keyring)
  - [last_err](https://sn0int.readthedocs.io/en/latest/reference.html#last-err)
  - [pgp_pubkey](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey)
  - [pgp_pubkey_armored](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey-armored)
  - [print](https://sn0int.readthedocs.io/en/latest/reference.html#print)
  - [psl_domain_from_dns_name](https://sn0int.readthedocs.io/en/latest/reference.html#psl-domain-from-dns-name)
  - [regex_find](https://sn0int.readthedocs.io/en/latest/reference.html#regex-find)
  - [regex_find_all](https://sn0int.readthedocs.io/en/latest/reference.html#regex-find-all)
  - [sleep](https://sn0int.readthedocs.io/en/latest/reference.html#sleep)
  - [sock_connect](https://sn0int.readthedocs.io/en/latest/reference.html#sock-connect)
  - [sock_send](https://sn0int.readthedocs.io/en/latest/reference.html#sock-send)
  - [sock_recv](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recv)
  - [sock_sendline](https://sn0int.readthedocs.io/en/latest/reference.html#sock-sendline)
  - [sock_recvline](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvline)
  - [sock_recvall](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvall)
  - [sock_recvline_contains](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvline-contains)
  - [sock_recvline_regex](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvline-regex)
  - [sock_recvn](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvn)
  - [sock_recvuntil](https://sn0int.readthedocs.io/en/latest/reference.html#sock-recvuntil)
  - [sock_sendafter](https://sn0int.readthedocs.io/en/latest/reference.html#sock-sendafter)
  - [sock_newline](https://sn0int.readthedocs.io/en/latest/reference.html#sock-newline)
  - [status](https://sn0int.readthedocs.io/en/latest/reference.html#status)
  - [stdin_readline](https://sn0int.readthedocs.io/en/latest/reference.html#stdin-readline)
  - [url_decode](https://sn0int.readthedocs.io/en/latest/reference.html#url-decode)
  - [url_encode](https://sn0int.readthedocs.io/en/latest/reference.html#url-encode)
  - [url_escape](https://sn0int.readthedocs.io/en/latest/reference.html#url-escape)
  - [url_join](https://sn0int.readthedocs.io/en/latest/reference.html#url-join)
  - [url_parse](https://sn0int.readthedocs.io/en/latest/reference.html#url-parse)
  - [url_unescape](https://sn0int.readthedocs.io/en/latest/reference.html#url-unescape)
  - [utf8_decode](https://sn0int.readthedocs.io/en/latest/reference.html#utf8-decode)
  - [x509_parse_pem](https://sn0int.readthedocs.io/en/latest/reference.html#x509-parse-pem)

## License

GPLv3+
