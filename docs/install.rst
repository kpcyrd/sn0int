Installation
============

If available, please prefer the package shipped by your linux distribution.

Archlinux
---------

.. code-block:: bash

    $ pacman -S sn0int

Mac OSX
-------

.. code-block:: bash

    $ brew install sn0int

Debian testing/Debian sid/Kali
------------------------------

Note that debian `doesn't ship the geoip2-database
<https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=757723>`_ so we're going to
download them automatically during the first run.

.. code-block:: bash

    $ apt install build-essential cargo libsqlite3-dev libseccomp-dev publicsuffix
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

Ubuntu/Debian stable
--------------------

cargo in the repos is too old and the build is `going to fail
<https://github.com/kpcyrd/sn0int/issues/68>`_. You should either install the
most recent rust version with `rustup <https://rustup.rs/>`_ or use the docker
instructions instead.

.. code-block:: bash

    $ apt install build-essential libsqlite3-dev libseccomp-dev publicsuffix
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

Docker
------

.. code-block:: bash

    $ docker run --rm --init -it -v $PWD/.cache:/cache -v $PWD/.data:/data kpcyrd/sn0int

Alpine
------

.. code-block:: bash

    $ apk add --no-cache sqlite-dev libseccomp-dev cargo
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

OpenBSD
-------

.. code-block:: bash

    $ pkg_add sqlite3
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

Windows
-------

This is not recommended and only passively maintained. Please prefer linux in a virtual machine if needed.

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ curl -fsS --retry 3 --retry-connrefused -o sqlite3.zip https://sqlite.org/2017/sqlite-dll-win64-x64-3160200.zip
    $ 7z e sqlite3.zip -y
    $ "C:\\Program Files (x86)\\Microsoft Visual Studio 14.0\\VC\\bin\\lib.exe" /def:sqlite3.def /OUT:sqlite3.lib /machine:x64
    $ export SQLITE3_LIB_DIR="$TRAVIS_BUILD_DIR"
    $ cargo install -f --path .
