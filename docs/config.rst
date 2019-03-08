Configuration
=============

This section documents the config file. By default this file does not exist and
a default configuration is used instead.

Linux/BSD
    ``~/.config/sn0int.toml``

OSX
    ``~/Library/Preferences/sn0int.toml``

Windows
    ``%APPDATA%/sn0int.toml``

[core]
------

``registry``
    Configure the registry you want to use. Defaults to ``https://sn0int.com``.
``no-autoupdate``
    sn0int is going to check if your modules are outdated during startout once
    a week. Set this option to ``true`` to disable this.

[namespaces]
------------------

By default sn0int modules are assumed to be installed from the registry. You
may want to keep a local directory with private modules, especially during
development. You can configure a folder that contains modules that aren't
managed by sn0int by adding a namespace section to the config file::

    [namespaces]
    foo = "/opt/sn0int/foo"
    bar = "~/repos/a/b/c/sn0int-modules"

This is going to load modules from these two folders and register them in the
``foo`` and ``bar`` namespace.

Note that sn0int is also going to assume that symlinks in
``~/.local/share/sn0int/modules`` and folders containing a ``.git`` folder are
externally managed.

[network]
---------

To enable a proxy, add the following to your config file::

    [network]
    proxy = "127.0.0.1:9050"

This forces everything through tor (or any other socks5 proxy) and restricts
all other functions that depend on the network. For example the ``dns``
function is fully disabled if a proxy is configured.
