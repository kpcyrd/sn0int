# sn0int [![crates.io][crates-img]][crates] [![Documentation Status][docs-img]][docs] [![irc.hackint.org:6697/#sn0int][irc-img]][irc] [![@sn0int][twitter-img]][twitter] [![@sn0int@chaos.social][mastodon-img]][mastodon] [![registry status][registry-img]][registry]

[crates-img]:   https://img.shields.io/crates/v/sn0int.svg
[crates]:       https://crates.io/crates/sn0int
[docs-img]:     https://readthedocs.org/projects/sn0int/badge/?version=latest
[docs]:         https://sn0int.readthedocs.io/en/latest/?badge=latest
[irc-img]:      https://img.shields.io/badge/hackint-%23sn0int-blue.svg
[irc]:          https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int
[twitter-img]:  https://img.shields.io/badge/twitter-@sn0int-blue.svg
[twitter]:      https://twitter.com/sn0int
[mastodon-img]: https://img.shields.io/badge/mastodon-chaos.social-blue.svg
[mastodon]:     https://chaos.social/@sn0int
[registry-img]: https://img.shields.io/website/https/sn0int.com.svg?label=registry
[registry]:     https://sn0int.com/

sn0int (pronounced [`/snoɪnt/`][ipa]) is a semi-automatic OSINT framework and
package manager. It's used by IT security professionals, bug bounty hunters,
law enforcement agencies and in security awareness trainings to gather
intelligence about a given target or about yourself. sn0int is enumerating
attack surface by semi-automatically processing public information and mapping
the results in a unified format for followup investigations.

[ipa]: http://ipa-reader.xyz/?text=sno%C9%AAnt

Among other things, sn0int is currently able to:

- Harvest subdomains from certificate transparency logs and passive dns
- Mass resolve collected subdomains and scan for http or https services
- Enrich ip addresses with asn and geoip info
- Harvest emails from pgp keyservers and whois
- Discover compromised logins in breaches
- Find somebody's profiles across the internet
- Enumerate local networks with unique techniques like passive arp
- Gather information about phonenumbers
- Harvest activity and images from social media profiles
- Basic image processing

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource. None of the investigations listed above are hardcoded
in the source, instead they are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own instead of pull-requesting
them into the sn0int codebase.

For questions and support join us on IRC: [irc.hackint.org:6697/#sn0int](https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int)

[![asciicast](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB.svg)](https://asciinema.org/a/shZ3TVY1o0opGFln3Oi2DAMCB)

## Installation

<a href="https://repology.org/project/sn0int/versions"><img align="right" src="https://repology.org/badge/vertical-allrepos/sn0int.svg" alt="Packaging status"></a>

Archlinux

    pacman -S sn0int

Mac OSX

    brew install sn0int

Debian/Ubuntu/Kali

There are prebuilt packages signed by a debian maintainer:

    sudo apt install curl sq
    curl -sSf https://apt.vulns.sexy/kpcyrd.pgp | sq dearmor | sudo tee /etc/apt/trusted.gpg.d/apt-vulns-sexy.gpg > /dev/null
    echo deb http://apt.vulns.sexy stable main | sudo tee /etc/apt/sources.list.d/apt-vulns-sexy.list
    apt update

Docker

    docker run --rm --init -it -v "$PWD/.cache:/cache" -v "$PWD/.data:/data" kpcyrd/sn0int

Alpine

    apk add sn0int

OpenBSD

    pkg_add sn0int

Gentoo

    layman -a pentoo
    emerge --ask net-analyzer/sn0int

NixOS

    nix-env -i sn0int

For everything else please have a look at the [detailed list][1].

[1]: https://sn0int.readthedocs.io/en/latest/install.html

## Getting started

- [Installation](https://sn0int.readthedocs.io/en/latest/install.html)
  - [Archlinux](https://sn0int.readthedocs.io/en/latest/install.html#archlinux)
  - [Mac OSX](https://sn0int.readthedocs.io/en/latest/install.html#mac-osx)
  - [Debian &gt;= bullseye, Ubuntu &gt;= 20.04, Kali](https://sn0int.readthedocs.io/en/latest/install.html#debian-bullseye-ubuntu-20-04-kali)
  - [Debian &lt;= buster, Ubuntu &lt;= 19.10](https://sn0int.readthedocs.io/en/latest/install.html#debian-buster-ubuntu-19-10)
  - [Fedora/CentOS/Redhat](https://sn0int.readthedocs.io/en/latest/install.html#fedora-centos-redhat)
  - [Docker](https://sn0int.readthedocs.io/en/latest/install.html#docker)
  - [Alpine](https://sn0int.readthedocs.io/en/latest/install.html#alpine)
  - [OpenBSD](https://sn0int.readthedocs.io/en/latest/install.html#openbsd)
  - [Gentoo](https://sn0int.readthedocs.io/en/latest/install.html#gentoo)
  - [NixOS](https://sn0int.readthedocs.io/en/latest/install.html#nixos)
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
- [Autonoscope](https://sn0int.readthedocs.io/en/latest/autonoscope.html)
  - [Domains](https://sn0int.readthedocs.io/en/latest/autonoscope.html#domains)
  - [IPs](https://sn0int.readthedocs.io/en/latest/autonoscope.html#ips)
  - [URLs](https://sn0int.readthedocs.io/en/latest/autonoscope.html#urls)
- [Writing your first module](https://sn0int.readthedocs.io/en/latest/scripting.html)
  - [Creating a repository](https://sn0int.readthedocs.io/en/latest/scripting.html#creating-a-repository)
  - [Publish your module](https://sn0int.readthedocs.io/en/latest/scripting.html#publish-your-module)
  - [Publish your repo](https://sn0int.readthedocs.io/en/latest/scripting.html#publish-your-repo)
  - [Reading data from stdin](https://sn0int.readthedocs.io/en/latest/scripting.html#reading-data-from-stdin)
- [Database](https://sn0int.readthedocs.io/en/latest/database.html)
  - [db_add](https://sn0int.readthedocs.io/en/latest/database.html#db-add)
  - [db_add_ttl](https://sn0int.readthedocs.io/en/latest/database.html#db-add-ttl)
  - [db_activity](https://sn0int.readthedocs.io/en/latest/database.html#db-activity)
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
  - [CryptoAddrs](https://sn0int.readthedocs.io/en/latest/structs.html#cryptoaddrs)
  - [Activity](https://sn0int.readthedocs.io/en/latest/structs.html#activity)
  - [Relations](https://sn0int.readthedocs.io/en/latest/structs.html#relations)
    - [subdomain_ipaddr](https://sn0int.readthedocs.io/en/latest/structs.html#subdomain-ipaddr)
    - [network_device](https://sn0int.readthedocs.io/en/latest/structs.html#network-device)
    - [breach_email](https://sn0int.readthedocs.io/en/latest/structs.html#breach-email)
- [Activity](https://sn0int.readthedocs.io/en/latest/activity.html)
  - [Anatomy of an event](https://sn0int.readthedocs.io/en/latest/activity.html#anatomy-of-an-event)
  - [Logging events](https://sn0int.readthedocs.io/en/latest/activity.html#logging-events)
  - [Querying events](https://sn0int.readthedocs.io/en/latest/activity.html#querying-events)
  - [Visualization](https://sn0int.readthedocs.io/en/latest/activity.html#visualization)
- [Notifications](https://sn0int.readthedocs.io/en/latest/notifications.html)
  - [Receiving notifications](https://sn0int.readthedocs.io/en/latest/notifications.html#receiving-notifications)
    - [Telegram](https://sn0int.readthedocs.io/en/latest/notifications.html#telegram)
    - [Pushover](https://sn0int.readthedocs.io/en/latest/notifications.html#pushover)
    - [Discord](https://sn0int.readthedocs.io/en/latest/notifications.html#discord)
    - [Signal](https://sn0int.readthedocs.io/en/latest/notifications.html#signal)
    - [Writing your own module](https://sn0int.readthedocs.io/en/latest/notifications.html#writing-your-own-module)
  - [Setting up notification rules](https://sn0int.readthedocs.io/en/latest/notifications.html#setting-up-notification-rules)
  - [Testing notifications](https://sn0int.readthedocs.io/en/latest/notifications.html#testing-notifications)
  - [Running sn0int automatically](https://sn0int.readthedocs.io/en/latest/notifications.html#running-sn0int-automatically)
    - [Monitors](https://sn0int.readthedocs.io/en/latest/notifications.html#monitors)
    - [Timers](https://sn0int.readthedocs.io/en/latest/notifications.html#timers)
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
  - [db_activity](https://sn0int.readthedocs.io/en/latest/reference.html#db-activity)
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
  - [http_fetch](https://sn0int.readthedocs.io/en/latest/reference.html#http-fetch)
  - [http_fetch_json](https://sn0int.readthedocs.io/en/latest/reference.html#http-fetch-json)
  - [img_load](https://sn0int.readthedocs.io/en/latest/reference.html#img-load)
  - [img_exif](https://sn0int.readthedocs.io/en/latest/reference.html#img-exif)
  - [img_ahash](https://sn0int.readthedocs.io/en/latest/reference.html#img-ahash)
  - [img_dhash](https://sn0int.readthedocs.io/en/latest/reference.html#img-dhash)
  - [img_phash](https://sn0int.readthedocs.io/en/latest/reference.html#img-phash)
  - [img_nudity](https://sn0int.readthedocs.io/en/latest/reference.html#img-nudity)
  - [info](https://sn0int.readthedocs.io/en/latest/reference.html#info)
  - [intval](https://sn0int.readthedocs.io/en/latest/reference.html#intval)
  - [json_decode](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode)
  - [json_decode_stream](https://sn0int.readthedocs.io/en/latest/reference.html#json-decode-stream)
  - [json_encode](https://sn0int.readthedocs.io/en/latest/reference.html#json-encode)
  - [key_trunc_pad](https://sn0int.readthedocs.io/en/latest/reference.html#key-trunc-pad)
  - [keyring](https://sn0int.readthedocs.io/en/latest/reference.html#keyring)
  - [last_err](https://sn0int.readthedocs.io/en/latest/reference.html#last-err)
  - [md5](https://sn0int.readthedocs.io/en/latest/reference.html#md5)
  - [mqtt_connect](https://sn0int.readthedocs.io/en/latest/reference.html#mqtt-connect)
  - [mqtt_subscribe](https://sn0int.readthedocs.io/en/latest/reference.html#mqtt-subscribe)
  - [mqtt_recv](https://sn0int.readthedocs.io/en/latest/reference.html#mqtt-recv)
  - [mqtt_ping](https://sn0int.readthedocs.io/en/latest/reference.html#mqtt-ping)
  - [pgp_pubkey](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey)
  - [pgp_pubkey_armored](https://sn0int.readthedocs.io/en/latest/reference.html#pgp-pubkey-armored)
  - [print](https://sn0int.readthedocs.io/en/latest/reference.html#print)
  - [psl_domain_from_dns_name](https://sn0int.readthedocs.io/en/latest/reference.html#psl-domain-from-dns-name)
  - [ratelimit_throttle](https://sn0int.readthedocs.io/en/latest/reference.html#ratelimit-throttle)
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
  - [sn0int_time](https://sn0int.readthedocs.io/en/latest/reference.html#sn0int-time)
  - [sn0int_time_from](https://sn0int.readthedocs.io/en/latest/reference.html#sn0int-time-from)
  - [sn0int_version](https://sn0int.readthedocs.io/en/latest/reference.html#sn0int-version)
  - [sock_connect](https://sn0int.readthedocs.io/en/latest/reference.html#sock-connect)
  - [sock_upgrade_tls](https://sn0int.readthedocs.io/en/latest/reference.html#sock-upgrade-tls)
  - [sock_options](https://sn0int.readthedocs.io/en/latest/reference.html#sock-options)
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
  - [sodium_secretbox_open](https://sn0int.readthedocs.io/en/latest/reference.html#sodium-secretbox-open)
  - [status](https://sn0int.readthedocs.io/en/latest/reference.html#status)
  - [stdin_readline](https://sn0int.readthedocs.io/en/latest/reference.html#stdin-readline)
  - [stdin_read_to_end](https://sn0int.readthedocs.io/en/latest/reference.html#stdin-read-to-end)
  - [str_find](https://sn0int.readthedocs.io/en/latest/reference.html#str-find)
  - [str_replace](https://sn0int.readthedocs.io/en/latest/reference.html#str-replace)
  - [strftime](https://sn0int.readthedocs.io/en/latest/reference.html#strftime)
  - [strptime](https://sn0int.readthedocs.io/en/latest/reference.html#strptime)
  - [strval](https://sn0int.readthedocs.io/en/latest/reference.html#strval)
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
  - [ws_connect](https://sn0int.readthedocs.io/en/latest/reference.html#ws-connect)
  - [ws_options](https://sn0int.readthedocs.io/en/latest/reference.html#ws-options)
  - [ws_recv_text](https://sn0int.readthedocs.io/en/latest/reference.html#ws-recv-text)
  - [ws_recv_binary](https://sn0int.readthedocs.io/en/latest/reference.html#ws-recv-binary)
  - [ws_recv_json](https://sn0int.readthedocs.io/en/latest/reference.html#ws-recv-json)
  - [ws_send_text](https://sn0int.readthedocs.io/en/latest/reference.html#ws-send-text)
  - [ws_send_binary](https://sn0int.readthedocs.io/en/latest/reference.html#ws-send-binary)
  - [ws_send_json](https://sn0int.readthedocs.io/en/latest/reference.html#ws-send-json)
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
