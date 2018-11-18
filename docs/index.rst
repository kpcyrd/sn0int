sn0int
======

sn0int is an OSINT framework and package manager. It's purpose is
semi-automatically processing public information to enumerate attack surface.
sn0int itself is only providing an engine that can be extended with scripts.

sn0int is heavily inspired by recon-ng, but takes a few different design
approaches. We've tried to correct some limitations in the database design and
also addressed the modularity problem:

Instead of downloading and reviewing python scripts that have full access to
your system, sn0int is executing modules in a lua sandbox. Modules can be
published to the sn0int registry and then installed by users. This means that
you don't have to send pull requests to sn0int to add a module and updates can
be shipped much faster.

Getting Started
---------------

.. toctree::
   :maxdepth: 3
   :glob:

   install
   usage
   scripting
   database
