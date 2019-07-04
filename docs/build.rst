Build from source
=================

It's generally recommended to `install a package <install.html>`_ if available.
This section is about building the binary from git.

Install dependencies
--------------------

You need a recent rust compiler. It's usually recommended to install a rust
compiler with `rustup <https://rustup.rs/>`_, but if you're system ships the
most recent compiler in a package that works too. Note that some systems aren't
fully supported by rustup (like OpenBSD and alpine) and you need to install
rust from a package in that case.

Archlinux
~~~~~~~~~

.. code-block:: bash

    $ pacman -S geoip2-database libseccomp publicsuffix-list sqlite

Mac OSX
~~~~~~~

None.

Debian/Ubuntu/Kali
~~~~~~~~~~~~~~~~~~

.. code-block:: bash

    $ apt install build-essential libsqlite3-dev libseccomp-dev publicsuffix

.. warning::
   On a debian based system make sure you've installed rust with rustup.

Alpine
~~~~~~

.. code-block:: bash

    $ apk add sqlite-dev libseccomp-dev

OpenBSD
~~~~~~~

.. code-block:: bash

    $ pkg_add sqlite3 geolite2-city geolite2-asn

Gentoo
~~~~~~

.. code-block:: bash

    emerge --ask sys-libs/libseccomp dev-db/sqlite

Windows
~~~~~~~

You don't need to install any dependencies on windows, but you need to use a
different build command in the next section.

Building
--------

After all dependencies have been installed, simply build the binary:

.. code-block:: bash

    $ cargo build --release

Note that you need a different command on windows:

.. code-block:: bash

    $ cargo build --release --features=sqlite-bundled

After the build finished the binary is located at ``target/release/sn0int``.
