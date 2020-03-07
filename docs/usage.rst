Running your first investigation
================================

This page is going to guide you through the process of setting up your
environment and running your first investigation.

Installing the default modules
------------------------------

By default, sn0int doesn't have any modules installed. If you start up sn0int
it's going to download some files that it needs and then suggests to install a
number of recommended modules::

    $ sn0int

                       ___/           .
         ____ , __   .'  /\ ` , __   _/_
        (     |'  `. |  / | | |'  `.  |
        `--.  |    | |,'  | | |    |  |
       \___.' /    | /`---' / /    |  \__/

            osint | recon | security
          irc.hackint.org:6697/#sn0int

    [+] Connecting to database
    [+] Downloading public suffix list
    [+] Downloading "GeoLite2-City.mmdb"
    [+] Downloading "GeoLite2-ASN.mmdb"
    [+] Loaded 0 modules
    [*] No modules found, run pkg quickstart to install default modules
    [sn0int][default] >

Typing ``pkg quickstart`` is going to get you a fair number of featured modules::

    [sn0int][default] > pkg quickstart
    [+] Installing kpcyrd/asn
    [+] Installing kpcyrd/ctlogs
    [+] Installing kpcyrd/dns-resolve
    [+] Installing kpcyrd/geoip
    [+] Installing kpcyrd/hackertarget-subdomains
    [+] Installing kpcyrd/otx-subdomains
    [+] Installing kpcyrd/passive-spider
    [+] Installing kpcyrd/pgp-keyserver
    [+] Installing kpcyrd/threatminer-ipaddr
    [+] Installing kpcyrd/threatminer-subdomains
    [+] Installing kpcyrd/url-scan
    [+] Installing kpcyrd/waybackurls
    [+] Loaded 12 modules
    [sn0int][default] >

Adding something to scope
-------------------------

You probably want to separate your investigations so you should select a
workspace where your results should go::

    [sn0int][default] > workspace demo
    [+] Connecting to database
    [sn0int][demo] >

Next, we have to start somewhere and add the first entity to our scope::

    [sn0int][demo] > add domain
    Domain: example.com
    [sn0int][demo] >

.. note::
   There is a concept of a domain vs a subdomain. We are referring to a domain
   as everything that is a subdomain of a `public suffix`_. For example, .com
   is a public suffix, which makes example.com a domain in sn0int terms. Every
   subdomain of that, like www.example.com, is referred to as a subdomain.

   Note that example.com can be added as a subdomain as well since it can hold
   records. In that case, example.com is both the name of the dns zone, while
   also being an entity in that zone.

.. _public suffix: https://publicsuffix.org/

You can confirm this by running a select on the domains we now have::

    [sn0int][demo] > select domains
    #1, "example.com"
    [sn0int][demo] >

Something we don't need right now, but is going to be useful later on is the
ability to filter your entities::

    [sn0int][demo] > select domains where id=1
    #1, "example.com"
    [sn0int][demo] >
    [sn0int][demo] > select domains where value like %.com
    #1, "example.com"
    [sn0int][demo] >
    [sn0int][demo] > select domains where ( value like e% and value like %m ) or false
    #1, "example.com"
    [sn0int][demo] >

.. note::
   Almost all entities have a ``value`` column that holds the primary value of
   the entity.

Running a module
----------------

Now that we have something to get started with, we can run our first module.
First lets list all modules we have::

    [sn0int][demo] > pkg list
    kpcyrd/asn (0.1.0)
        Run a asn lookup for an ip address
    kpcyrd/ctlogs (0.1.0)
        Query certificate transparency logs to discover subdomains
    kpcyrd/dns-resolve (0.1.0)
        Query subdomains to discovery ip addresses and verify the record is visible
    kpcyrd/geoip (0.1.0)
        Run a geoip lookup for an ip address
    kpcyrd/hackertarget-subdomains (0.1.0)
        Query hackertarget for subdomains of a domain
    kpcyrd/otx-subdomains (0.1.0)
        Query alienvault otx passive dns for subdomains of a domain
    kpcyrd/passive-spider (0.1.0)
        Scrape known http responses for urls
    kpcyrd/pgp-keyserver (0.1.0)
        Query pgp keyserver for email addresses
    kpcyrd/threatminer-ipaddr (0.1.0)
        Query ThreatMiner passive dns for subdomains of an ip address
    kpcyrd/threatminer-subdomains (0.1.0)
        Query ThreatMiner passive dns for subdomains of a domain
    kpcyrd/url-scan (0.1.0)
        Scan subdomains for websites
    kpcyrd/waybackurls (0.1.0)
        Discover subdomains from wayback machine
    [sn0int][demo] >

Let's start by querying certificate transparency logs::

    [sn0int][demo] > use ctlogs
    [sn0int][demo][kpcyrd/ctlogs] > run
    [*] "example.com"                                     : Subdomain: "www.example.com"
    [*] "example.com"                                     : Subdomain: "m.example.com"
    [*] "example.com"                                     : Subdomain: "dev.example.com"
    [*] "example.com"                                     : Subdomain: "products.example.com"
    [*] "example.com"                                     : Subdomain: "support.example.com"
    [+] Finished kpcyrd/ctlogs
    [sn0int][demo][kpcyrd/ctlogs] >

Looks like we've discovered some subdomains here. It might be tempting to throw
some of them in a browser but hold on, there's a more efficient way to approach
this.

.. hint::
   You can run the modules concurrently with ``run -j3``.

Running followup modules on the results
---------------------------------------

A lot of time has been spent on the database part. While it sort of feels like
a no-sql database we are actually enforcing a schema for a reason instead of
just using generic dictionaries and calling it a day.

It's crucial that entities created by one module can be picked up by another
module, like LEGOs. Let's continue with a module to query the dns records::

    [sn0int][demo][kpcyrd/ctlogs] > use dns-resolve
    [sn0int][demo][kpcyrd/dns-resolve] > run
    [*] "www.example.com"                                 : Updating "www.example.com" (resolvable => true)
    [*] "www.example.com"                                 : IpAddr: 93.184.216.34
    [*] "www.example.com"                                 : "www.example.com" -> 93.184.216.34
    [*] "m.example.com"                                   : Updating "m.example.com" (resolvable => false)
    [*] "dev.example.com"                                 : Updating "dev.example.com" (resolvable => false)
    [*] "products.example.com"                            : Updating "products.example.com" (resolvable => false)
    [*] "support.example.com"                             : Updating "support.example.com" (resolvable => false)
    [+] Finished kpcyrd/dns-resolve
    [sn0int][demo][kpcyrd/dns-resolve] >

.. TODO: mention https://github.com/kpcyrd/sn0int/issues/27

Two things happened here: We've discovered some IP addresses and added them to
scope, and we also updated our subdomain entities with new information, since
we now know which of them are resolvable and which aren't.

Let's run the next module, which is actually going to check for websites on
them, but let's only target subdomains that we know are resolvable::

    [sn0int][demo][kpcyrd/dns-resolve] > use url-scan
    [sn0int][demo][kpcyrd/url-scan] > target
    #1, "www.example.com"
        93.184.216.34
    #2, "m.example.com"
    #3, "dev.example.com"
    #4, "products.example.com"
    #5, "support.example.com"
    [sn0int][demo][kpcyrd/url-scan] > target where resolvable
    [+] 1 entities selected
    [sn0int][demo][kpcyrd/url-scan] > target
    #1, "www.example.com"
        93.184.216.34
    [sn0int][demo][kpcyrd/url-scan] >

We can both preview and limit the targets that are going to be passed to the
module with the target command. Once we are satisfied with our selection we can
run this module::

    [sn0int][demo][kpcyrd/url-scan] > run
    [*] "www.example.com"                                 : Url: "http://www.example.com/" (200)
    [*] "www.example.com"                                 : Url: "https://www.example.com/" (200)
    [+] Finished kpcyrd/url-scan
    [sn0int][demo][kpcyrd/url-scan] >

We've now probed both port 80 and port 443 for each subdomain and found two
http responses this way. If you want a list of urls you may want to visit in
your browser can now query them::

    [sn0int][demo][kpcyrd/url-scan] > select urls
    #1, "http://www.example.com/" (200)
    #2, "https://www.example.com/" (200)
    [sn0int][demo][kpcyrd/url-scan] >

Unscoping entities
------------------

Something you are going to run into is that modules are too greedy and add
things to the scope we are not interested in. You can delete them using the
delete command, but those are likely picked up by a module again.

What you can do instead is setting a flag on an entity that removes it from
our scope. This is done using the noscope command::

    [sn0int][demo] > use ctlogs
    [sn0int][demo][kpcyrd/ctlogs] > target
    #1, "example.com"
    [sn0int][demo][kpcyrd/ctlogs] > add domain
    Domain: google.com
    [sn0int][demo][kpcyrd/ctlogs] > target
    #1, "example.com"
    #2, "google.com"
    [sn0int][demo][kpcyrd/ctlogs] > noscope domains where value=google.com
    [+] Updated 1 rows
    [sn0int][demo][kpcyrd/ctlogs] > target
    #1, "example.com"
    [sn0int][demo][kpcyrd/ctlogs] >

Entities that are unscoped are automatically ignored by all modules.

You can reverse this using the scope command::

    [sn0int][demo][kpcyrd/ctlogs] > target
    #1, "example.com"
    [sn0int][demo][kpcyrd/ctlogs] > scope domains where true
    [+] Updated 2 rows
    [sn0int][demo][kpcyrd/ctlogs] > target
    #1, "example.com"
    #2, "google.com"
    [sn0int][demo][kpcyrd/ctlogs] >

.. hint::
   All entities have this field, you can refer to it in queries using
   ``unscoped=1``.
