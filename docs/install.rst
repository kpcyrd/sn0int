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

Debian >= bookwork, Ubuntu >= 22.10, Kali
-----------------------------------------

There are prebuilt packages signed by a debian maintainer:

.. code-block:: bash

    $ sudo apt install curl sq
    $ curl -sSf https://apt.vulns.sexy/kpcyrd.pgp | sq keyring filter -B --handle 64B13F7117D6E07D661BBCE0FE763A64F5E54FD6 | sudo tee /etc/apt/trusted.gpg.d/apt-vulns-sexy.gpg > /dev/null
    $ echo deb http://apt.vulns.sexy stable main | sudo tee /etc/apt/sources.list.d/apt-vulns-sexy.list
    $ apt update
    $ apt install sn0int

Debian <= bullseye, Ubuntu <= 22.04
-----------------------------------

There are prebuilt packages signed by a debian maintainer:

.. code-block:: bash

    $ sudo apt install curl sq
    $ curl -sSf https://apt.vulns.sexy/kpcyrd.pgp | sq dearmor | sudo tee /etc/apt/trusted.gpg.d/apt-vulns-sexy.gpg > /dev/null
    $ echo deb http://apt.vulns.sexy stable main | sudo tee /etc/apt/sources.list.d/apt-vulns-sexy.list
    $ apt update
    $ apt install sn0int

Fedora/CentOS/Redhat
--------------------

Using rust+cargo from the repos might work for you, but we only officially
support rust+cargo installed with `rustup <https://rustup.rs/>`_. Have a look
at the docker image as an alternative.

.. code-block:: bash

    $ dnf install @development-tools libsq3-devel libseccomp-devel libsodium-devel publicsuffix-list
    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .

Docker
------

.. code-block:: bash

    $ docker run --rm --init -it -v "$PWD/.cache:/cache" -v "$PWD/.data:/data" kpcyrd/sn0int

Alpine
------

.. code-block:: bash

    $ apk add sn0int

OpenBSD
-------

.. code-block:: bash

    $ pkg_add sn0int

Gentoo
------

.. code-block:: bash

    $ layman -a pentoo
    $ emerge --ask net-analyzer/sn0int

NixOS
-----

.. code-block:: bash

    $ nix-env -i sn0int

Windows
-------

This is not recommended and only passively maintained. Please prefer linux in a
virtual machine if needed.

Make sure rust is installed and setup.

.. code-block:: bash

    $ git clone https://github.com/kpcyrd/sn0int.git
    $ cd sn0int
    $ cargo install -f --path .
