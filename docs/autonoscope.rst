Autonoscope
===========

Instead of manually unscoping everything you can also define so called
autonoscope rules. Those are executed from most specific to least specific and
the first match wins. If no rule matches, the default is in-scope::

    [sn0int][demo] > # add the domain first
    [sn0int][demo] > # this is necessary because we only want to partially unscope example.com
    [sn0int][demo] > add domain example.com
    [sn0int][demo] > 
    [sn0int][demo] > # automatically noscope all subdomains
    [sn0int][demo] > autonoscope add domain example.com
    [sn0int][demo] > # except subdomains of prod.example.com 
    [sn0int][demo] > autoscope add domain prod.example.com
    [sn0int][demo] > 
    [sn0int][demo] > autonoscope list 
      scope domain "prod.example.com"
    noscope domain "example.com"
    [sn0int][demo] > 
    [sn0int][demo] > # this is going to be out-of-scope
    [sn0int][demo] > add subdomain www.example.com
    [sn0int][demo] > # this is going to be in-scope
    [sn0int][demo] > add subdomain db.prod.example.com
    [sn0int][demo] > 
    [sn0int][demo] > select subdomains 
    #1, "www.example.com"
    #2, "db.prod.example.com"
    [sn0int][demo] > select subdomains where unscoped=0
    #2, "db.prod.example.com"
    [sn0int][demo] > select subdomains where unscoped=1
    #1, "www.example.com"
    [sn0int][demo] > 

Domains
-------

Autonoscope rules for domains are applied to the following structs:

- domains
- subdomains
- urls

Example rules::

    autonoscope add domain example.com
    autonoscope add domain staging.example.com
    autonoscope add domain com
    autonoscope add domain .

IPs
---

Autonoscope rules for IPs are applied to the following structs:

- ipaddrs
- netblocks
- ports

Example rules::

    autonoscope add ip 0.0.0.0/0
    autonoscope add ip ::/0
    autonoscope add ip 192.168.0.0/16
    autonoscope add ip 10.13.33.37/32

URLs
----

Autonoscope rules for urls are applied to the following structs:

- urls

Note that these rules are specific to a certain origin (like
``https://example.com``) and are used to filter paths.

Example rules::

    autonoscope add url https://example.com/
    autonoscope add url https://example.com/admin/
    autonoscope add url https://example.com/a/b/c/d
