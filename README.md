# sn0int [![Build Status][travis-img]][travis] [![crates.io][crates-img]][crates] [![Documentation Status][docs-img]][docs] [![irc.hackint.org:6697/#sn0int][irc-img]][irc] [![@sn0int@chaos.social][mastodon-img]][mastodon] [![registry status][registry-img]][registry]

[travis-img]:   https://travis-ci.org/kpcyrd/sn0int.svg?branch=master
[travis]:       https://travis-ci.org/kpcyrd/sn0int
[crates-img]:   https://img.shields.io/crates/v/sn0int.svg
[crates]:       https://crates.io/crates/sn0int
[docs-img]:     https://readthedocs.org/projects/sn0int/badge/?version=latest
[docs]:         https://sn0int.readthedocs.io/en/latest/?badge=latest
[irc-img]:      https://img.shields.io/badge/hackint-%23sn0int-blue.svg
[irc]:          https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int
[mastodon-img]: https://img.shields.io/badge/mastodon-chaos.social-blue.svg
[mastodon]:     https://chaos.social/@sn0int
[registry-img]: https://img.shields.io/website/https/sn0int.com.svg?label=registry
[registry]:     https://sn0int.com/

sn0int is a semi-automatic OSINT framework and package manager. It was built
for IT security professionals and bug hunters to gather intelligence about a
given target or about yourself. sn0int is enumerating attack surface by
semi-automatically processing public information and mapping the results in a
unified format for followup investigations.

Among other things, sn0int is currently able to:

- Harvest subdomains from certificate transparency logs and passive dns
- Enrich ip addresses with asn and geoip info
- Harvest emails from pgp keyservers and whois
- Discover compromised logins in breaches
- Find somebody's profiles across the internet
- Enumerate local networks with unique techniques like passive arp
- Gather information about phonenumbers
- Harvest data and images from instagram profiles
- Scan images for nudity

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource. None of the investigations listed above are hardcoded
in the source, instead those are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own since you don't need to send a
pull request.

For questions and support join us on IRC: [irc.hackint.org:6697/#sn0int](https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int)

[![asciicast](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB.svg)](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB)

## Installation

Archlinux

    pacman -S sn0int

Mac OSX

    brew install sn0int

For everything else please have a look at the [detailed list][1].

[1]: https://sn0int.readthedocs.io/en/latest/install.html

## Getting started

- [Installation](https://sn0int.readthedocs.io/en/latest/install.html)
  - [Archlinux](https://sn0int.readthedocs.io/en/latest/install.html#archlinux)
  - [Mac OSX](https://sn0int.readthedocs.io/en/latest/install.html#mac-osx)
  - [Debian/Ubuntu/Kali](https://sn0int.readthedocs.io/en/latest/install.html#debian-ubuntu-kali)
  - [Docker](https://sn0int.readthedocs.io/en/latest/install.html#docker)
  - [Alpine](https://sn0int.readthedocs.io/en/latest/install.html#alpine)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/install.html#openbsd)
  - [Gentoo](https://sn0int.readthedocs.io/en/latest/install.html#gentoo)
  - [Windows](https://sn0int.readthedocs.io/en/latest/install.html#windows)
- [Build from source](https://sn0int.readthedocs.io/en/latest/build.html)
  - [Install dependencies](https://sn0int.readthedocs.io/en/latest/build.html#install-dependencies)
    - [Archlinux](https://sn0int.readthedocs.io/en/latest/build.html#archlinux)
    - [Mac OSX](https://sn0int.readthedocs.io/en/latest/build.html#mac-osx)
    - [Debian/Ubuntu/Kali](https://sn0int.readthedocs.io/en/latest/build.html#debian-ubuntu-kali)
    - [Alpine](https://sn0int.readthedocs.io/en/latest/build.html#alpine)
    - [OpenBSD](https://sn0int.readthedocs.io/en/latest/build.html#openbsd)
    - [Gentoo](https://sn0int.readthedocs.io/en/latest/build.html#gentoo)
    - [Windows](https://sn0int.readthedocs.io/en/latest/build.html#windows)
  - [Building](https://sn0int.readthedocs.io/en/latest/build.html#building)
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
- [Structs](https://sn0int.readthedocs.io/en/latest/structs.html)
  - [Domains](https://sn0int.readthedocs.io/en/latest/structs.html#domains)
  - [Subdomains](https://sn0int.readthedocs.io/en/latest/structs.html#subdomains)
  - [IpAddrs](https://sn0int.readthedocs.io/en/latest/structs.html#ipaddrs)
  - [URLs](https://sn0int.readthedocs.io/en/latest/structs.html#urls)
  - [Emails](https://sn0int.readthedocs.io/en/latest/structs.html#emails)
  - [Phonenumbers](https://sn0int.readthedocs.io/en/latest/structs.html#phonenumbers)
  - [Devices](https://sn0int.readthedocs.io/en/latest/structs.html#devices)
  - [Networks](https://sn0int.readthedocs.io/en/latest/structs.html#networks)
  - [Accounts](https://sn0int.readthedocs.io/en/latest/structs.html#accounts)
  - [Breaches](https://sn0int.readthedocs.io/en/latest/structs.html#breaches)
  - [Images](https://sn0int.readthedocs.io/en/latest/structs.html#images)
  - [Ports](https://sn0int.readthedocs.io/en/latest/structs.html#ports)
  - [Netblocks](https://sn0int.readthedocs.io/en/latest/structs.html#netblocks)
  - [Relations](https://sn0int.readthedocs.io/en/latest/structs.html#relations)
    - [subdomain_ipaddr](https://sn0int.readthedocs.io/en/latest/structs.html#subdomain-ipaddr)
    - [network_device](https://sn0int.readthedocs.io/en/latest/structs.html#network-device)
    - [breach_email](https://sn0int.readthedocs.io/en/latest/structs.html#breach-email)
- [Keyring](https://sn0int.readthedocs.io/en/latest/keyring.html)
  - [Managing the keyring](https://sn0int.readthedocs.io/en/latest/keyring.html#managing-the-keyring)
  - [Using access keys in scripts](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-in-scripts)
  - [Using access keys as source argument](https://sn0int.readthedocs.io/en/latest/keyring.html#using-access-keys-as-source-argument)
- [Configuration](https://sn0int.readthedocs.io/en/latest/config.html)
  - [\[core\]](https://sn0int.readthedocs.io/en/latest/config.html#core)
  - [\[namespaces\]](https://sn0int.readthedocs.io/en/latest/config.html#namespaces)
  - [\[network\]](https://sn0int.readthedocs.io/en/latest/config.html#network)
- [Sandbox](https://sn0int.readthedocs.io/en/latest/sandbox.html)
  - [Linux](https://sn0int.readthedocs.io/en/latest/sandbox.html#linux)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/sandbox.html#openbsd)
  - [IPC Protocol](https://sn0int.readthedocs.io/en/latest/sandbox.html#ipc-protocol)
  - [Limitations](https://sn0int.readthedocs.io/en/latest/sandbox.html#limitations)
  - [Diagnosing a sandbox failure](https://sn0int.readthedocs.io/en/latest/sandbox.html#diagnosing-a-sandbox-failure)
- [Function reference](https://sn0int.readthedocs.io/en/latest/reference.html)
  - [asn_lookup](https://sn0int.readthedocs.io/en/latest/reference.html#asn-lookup)
  - [base64_decode](https://sn0int.readthedocs.io/en/latest/reference.html#base64-decode)
  - [base64_encode](https://sn0int.readthedocs.io/en/latest/reference.html#base64-encode)
  - [base64_custom_decode](https://sn0int.readthedocs.io/en/latest/reference.html#base64-custom-decode)
  - [base64_custom_encode](https://sn0int.readthedocs.io/en/latest/reference.html#base64-custom-encode)
  - [base32_custom_decode](https://sn0int.readthedocs.io/en/latest/reference.html#base32-custom-decode)
  - [base32_custom_encode](https://sn0int.readthedocs.io/en/latest/reference.html#base32-custom-encode)
  - [clear_err](https://sn0int.readthedocs.io/en/latest/reference.html#clear-err)
  - [create_blob](https://sn0int.readthedocs.io/en/latest/reference.html#create-blob)
  - [datetime](https://sn0int.readthedocs.io/en/latest/reference.html#datetime)
  - [db_add](https://sn0int.readthedocs.io/en/latest/reference.html#db-add)
  - [db_add_ttl](https://sn0int.readthedocs.io/en/latest/reference.html#db-add-ttl)
  - [db_select](https://sn0int.readthedocs.io/en/latest/reference.html#db-select)
  - [db_update](https://sn0int.readthedocs.io/en/latest/reference.html#db-update)
  - [dns](https://sn0int.readthedocs.io/en/latest/reference.html#dns)
  - [error](https://sn0int.readthedocs.io/en/latest/reference.html#error)
  - [geoip_lookup](https://sn0int.readthedocs.io/en/latest/reference.html#geoip-lookup)
  - [hex](https://sn0int.readthedocs.io/en/latest/reference.html#hex)
  - [hmac_md5](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-md5)
  - [hmac_sha1](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-sha1)
  - [hmac_sha2_256](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-sha2-256)
  - [hmac_sha2_512](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-sha2-512)
  - [hmac_sha3_256](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-sha3-256)
  - [hmac_sha3_512](https://sn0int.readthedocs.io/en/latest/reference.html#hmac-sha3-512)
  - [html_select](https://sn0int.readthedocs.io/en/latest/reference.html#html-select)
  - [html_select_list](https://sn0int.readthedocs.io/en/latest/reference.html#html-select-list)
  - [http_mksession](https://sn0int.readthedocs.io/en/latest/reference.html#http-mksession)
  - [http_request](https://sn0int.readthedocs.io/en/latest/reference.html#http-request)
  - [http_send](https://sn0int.readthedocs.io/en/latest/reference.html#http-send)
  - [http_fetch_json](https://sn0int.readthedocs.io/en/latest/reference.html#http-fetch-json)
  - [img_load](https://sn0int.readthedocs.io/en/latest/reference.html#img-load)
  - [img_exif](https://sn0int.readthedocs.io/en/latest/reference.html#img-exif)
  - [img_nudity](https://sn0int.readthedocs.io/en/latest/reference.html#img-nudity)
  - [info](https://sn0int.readthedocs.io/en/latest/reference.html#info)
  - [json_decode](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode)
  - [json_decode_stream](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode-stream)
  - [json_encode](https://sn0int.readthedocs.io/en/latest/reference.html#json-encode)
  - [keyring](https://sn0int.readthedocs.io/en/latest/reference.html#keyring)
  - [last_err](https://sn0int.readthedocs.io/en/latest/reference.html#last-err)
  - [md5](https://sn0int.readthedocs.io/en/latest/reference.html#md5)
  - [pgp_pubkey](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey)
  - [pgp_pubkey_armored](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey-armored)
  - [print](https://sn0int.readthedocs.io/en/latest/reference.html#print)
  - [psl_domain_from_dns_name](https://sn0int.readthedocs.io/en/latest/reference.html#psl-domain-from-dns-name)
  - [regex_find](https://sn0int.readthedocs.io/en/latest/reference.html#regex-find)
  - [regex_find_all](https://sn0int.readthedocs.io/en/latest/reference.html#regex-find-all)
  - [semver_match](https://sn0int.readthedocs.io/en/latest/reference.html#semver-match)
  - [set_err](https://sn0int.readthedocs.io/en/latest/reference.html#set-err)
  - [sha1](https://sn0int.readthedocs.io/en/latest/reference.html#sha1)
  - [sha2_256](https://sn0int.readthedocs.io/en/latest/reference.html#sha2-256)
  - [sha2_512](https://sn0int.readthedocs.io/en/latest/reference.html#sha2-512)
  - [sha3_256](https://sn0int.readthedocs.io/en/latest/reference.html#sha3-256)
  - [sha3_512](https://sn0int.readthedocs.io/en/latest/reference.html#sha3-512)
  - [sleep](https://sn0int.readthedocs.io/en/latest/reference.html#sleep)
  - [sn0int_version](https://sn0int.readthedocs.io/en/latest/reference.html#sn0int-version)
  - [sock_connect](https://sn0int.readthedocs.io/en/latest/reference.html#sock-connect)
  - [sock_upgrade_tls](https://sn0int.readthedocs.io/en/latest/reference.html#sock-upgrade-tls)
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
  - [strftime](https://sn0int.readthedocs.io/en/latest/reference.html#strftime)
  - [strptime](https://sn0int.readthedocs.io/en/latest/reference.html#strptime)
  - [time_unix](https://sn0int.readthedocs.io/en/latest/reference.html#time-unix)
  - [url_decode](https://sn0int.readthedocs.io/en/latest/reference.html#url-decode)
  - [url_encode](https://sn0int.readthedocs.io/en/latest/reference.html#url-encode)
  - [url_escape](https://sn0int.readthedocs.io/en/latest/reference.html#url-escape)
  - [url_join](https://sn0int.readthedocs.io/en/latest/reference.html#url-join)
  - [url_parse](https://sn0int.readthedocs.io/en/latest/reference.html#url-parse)
  - [url_unescape](https://sn0int.readthedocs.io/en/latest/reference.html#url-unescape)
  - [utf8_decode](https://sn0int.readthedocs.io/en/latest/reference.html#utf8-decode)
  - [warn](https://sn0int.readthedocs.io/en/latest/reference.html#warn)
  - [warn_once](https://sn0int.readthedocs.io/en/latest/reference.html#warn-once)
  - [x509_parse_pem](https://sn0int.readthedocs.io/en/latest/reference.html#x509-parse-pem)
  - [xml_decode](https://sn0int.readthedocs.io/en/latest/reference.html#xml-decode)
  - [xml_named](https://sn0int.readthedocs.io/en/latest/reference.html#xml-named)

## Rationale

This tool was written for companies to help them understand their attack
surface from a blackbox point of view. It's often difficult to understand that
something is easier to discover than some people assume, putting them at risk
of false security.

It's also designed to be useful for red team assessments and bug bounties,
which also help companies to identify weaknesses that could result in a
compromise.

Some functionality was written to do the same thing for individuals to raise
awareness about personal attack surface, privacy and how much data is publicly
available. These issues are often out of scope in bug bounties and sometimes by
design. We believe that blaming the user is the wrong approach and these issues
should be addressed at the root cause by the people designing those systems.

## License

GPLv3+
