Installation
============

If available, please prefer the package shipped by operating system. If your
operating system has a package but you're running on older version, please use
the `build from source <build.html>`_ instructions instead.

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

    $ docker run --rm --init -it -v "$PWD/.cache:/cache" -v "$PWD/.data:/data" kpcyrd/sn0int

Alpine
------

On alpine edge, with enabled testing repositories:

.. code-block:: bash

    $ apk add sn0int

OpenBSD
-------

On -current:

.. code-block:: bash

    $ pkg_add sn0int

Gentoo
------

.. code-block:: bash

    layman -f -o https://raw.githubusercontent.com/kpcyrd/overlay/master/overlay.xml -a kpcyrd-overlay
    emerge --ask net-analyzer/sn0int

Windows
-------

This is not recommended and only passively maintained. Please prefer linux in a
virtual machine if needed.

Make sure rust is installed and setup.

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path . --features=sqlite-bundled
