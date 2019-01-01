Configuration
=============

This file documents the config file at ``~/.config/sn0int.toml``. By default
this file does not exist and a default configuration is used instead.

Configuring a proxy
-------------------

To enable a proxy, add the following to your config file::

    [network]
    proxy = "127.0.0.1:9050"

This forces everything through tor and restricts all other functions that
depend on the network. For example the ``dns`` function is fully disabled if a
proxy is configured.
