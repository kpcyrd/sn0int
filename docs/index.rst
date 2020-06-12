sn0int
======

sn0int is a semi-automatic OSINT framework and package manager. It was built
for IT security professionals and bug hunters to gather intelligence about a
given target or about yourself. sn0int is enumerating attack surface by
semi-automatically processing public information and mapping the results in a
unified format for followup investigations.

Among other things, sn0int is currently able to:

- Harvest subdomains from certificate transparency logs
- Harvest subdomains from various passive dns logs
- Sift through subdomain results for publicly accessible websites
- Harvest emails from pgp keyservers
- Enrich ip addresses with ASN and geoip info
- Harvest subdomains from the wayback machine
- Gather information about phonenumbers
- Bruteforce interesting urls

sn0int is heavily inspired by recon-ng and maltego, but remains more flexible
and is fully opensource.  None of the investigations listed above are hardcoded
in the source, instead those are provided by modules that are executed in a
sandbox. You can easily extend sn0int by writing your own modules and share
them with other users by publishing them to the sn0int registry. This allows
you to ship updates for your modules on your own since you don't need to send a
pull request.

Join us on IRC: `irc.hackint.org:6697/#sn0int <https://webirc.hackint.org/#irc://irc.hackint.org/#sn0int>`_

Getting Started
---------------

.. toctree::
   :maxdepth: 3
   :glob:

   install
   build
   usage
   autonoscope
   scripting
   database
   structs
   activity
   notifications
   keyring
   config
   sandbox
   reference
