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

Debian/Ubuntu/Kali
------------------

Note that debian `doesn't ship the geoip2-database
<https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=757723>`_ so we're going to
download them automatically during the first run.

Using rust+cargo from the repos might work for you, but we only officially
support rust+cargo installed with `rustup <https://rustup.rs/>`_. Have a look
at the docker image as an alternative.

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

    $ pkg_add git cargo sqlite3 geolite2-city geolite2-asn
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

Windows
-------

This is not recommended and only passively maintained. Please prefer linux in a virtual machine if needed.

Make sure rust is installed and setup.

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path . --features=sqlite-bundled
