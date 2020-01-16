Structs
=======

This section describes all supported structs in depth. Please refer to this
section if in doubt about the correct usage of fields to ensure
interoperability between modules.

Domains
-------

Represents a registerable domain as defined by the `public suffix list
<https://publicsuffix.org/>`_. If in doubt check `psl_domain_from_dns_name
<reference.html#psl-domain-from-dns-name>`_.

``value``
  The domain name, like ``example.co.uk``.

Subdomains
----------

A subdomain of a `domain <#domains>`_. The depth is arbitrary, so
``foo.example.co.uk`` and ``foo.bar.example.co.uk`` are both valid subdomains
of ``example.co.uk``.

``value``
  The subdomain, like ``foo.bar.example.co.uk``.
``domain_id``
  The numeric id of a domain struct.
``resolvable``
  Whether the subdomain can be resolved to a A/AAAA record. nil if unknown.

IpAddrs
-------

An ip address. Note that most of these fields are geoip related and an
approximation instead of an actual location.

``value``
    The ip address.
``family``
    The address family of the ip address, either ``4`` or ``6``.
``continent``
    The continent associated with this ip address.
``continent_code``
    The continent code of the ``continent`` field, eg ``NA``.
``country``
    The country associated with this ip address.
``country_code``
    The country code of the ``country`` field, eg ``US``.
``city``
    The city associated with this ip address.
``latitude``
    Latitude associated with this ip address.
``longitude``
    Longitude associated with this ip address.
``asn``
    The number of the autonomous system this ip belongs to.
``as_org``
    The organization of the autonomous system this ip belongs to.
``description``
    This field is sn0int internal if we have additional information about this
    ip address, for example technical identifiers from aws.
``reverse_dns``
    The reverse dns name setup for this ip address.

URLs
----

``subdomain_id``
    The numeric id of a subdomain struct.
``value``
    The url, including a schema, hostname and path.
``status``
    The http status code, like ``200``.
``body``
    The raw response body. This can be any mime type.
``online``
    Whether or not the url gives a http response (even if it's an error).
``title``
    The parsed ``<title>`` of the page, if available.
``redirect``
    If the server replied with a redirect, this is the url it redirected to.

Emails
------

``value``
    The email address.
``displayname``
    The display name of a given email address: ``this is the name <foo@example.com>``.
``valid``
    Whether that email address is valid or has been disabled.

Phonenumbers
------------

``value``
    The phone number in E.164 format (+491234567)
``name``
    An alias we can assign to this phone number. This alias is sn0int internal.
``valid``
    Whether the number is assigned to a customer.
``last_online``
    The last time this number has been online.
``country``
    The country this number is associated with.
``carrier``
    The name of the carrier this numer is registered with.
``line``
    The type of the phone number, can be ``landline``, ``mobile`` or ``voip``.
``is_ported``
    Whether this number has been ported to a different carrier.
``last_ported``
    The last time this number has been ported.
``caller_name``
    The name of the owner of the phone number.
``caller_type``
    The type of caller, eg ``business`` or ``consumer``.

Devices
-------

``value``
    The devices mac address or another identifier if needed.
``name``
    An alias we can assign to this device. This alias is sn0int internal.
``hostname``
    The hostname configured on the device.
``vendor``
    The hardware vendor of the device. This is usually derived from the mac
    address.
``last_seen``
    The last time we've observed the device somewhere.

Networks
--------

A wired or wireless network at a specific location that a device could be
connected to.

``value``
    The network name. This can be an ssid or any other identifier but should be
    unique.
``latitude``
    Latitude of the networks location.
``longitude``
    Longitude of the networks location.
``description``
    A human readable description in case the value is a technical identifier.

Accounts
--------

A users account or profile on a webservice, like github or instagram.

``service``
    The identifier of the service/website. It's recommended to use the websites
    domain for this as defined in `Domains`_.
``username``
    The users unique identifier, like the login name. If the login name is not
    known or the system doesn't use login names, use the email address instead.
``displayname``
    The users display name. This name is often not unique and may contain the
    users real name.
``email``
    The email address associated with the account.
``url``
    The url of the public profile if available.
``last_seen``
    The last time this account has been active/online.
``birthday``
    The users birthday set on the account.
``phonenumber``
    The phonenumber associated with the account.
``profile_pic``
    The blob identifier of the users current profile picture.

Breaches
--------

Either a breach of a specific website, a breach compilation or a breach
notification service.

``value``
    The name of the breach, breach compilation or notification service.

Images
------

``value``
    The id that identifies the blob. This id is deterministic based on file
    content.
``filename``
    This field is used if we have a well known filename for the content.
``mime``
    The image mimetype, like ``image/png`` or ``image/jpeg``.
``width``
    The width of the image.
``height``
    The height of the image.
``created``
    The date and time this image has been taken.
``latitude``
    Latitude this picture has been taken.
``longitude``
    Longitude this picture has been taken.
``nudity``
    A score that classifies nudity in this picture. The score goes from 0 to 2
    and is commonly calculated with ``img_nudity``. A score above 1 means
    nudity has been detected.
``ahash``
    The Mean (aHash) perceptual hash.
``dhash``
    The Gradient (dHash) perceptual hash.
``phash``
    The DCT (pHash) perceptual hash.

Ports
-----

The status of a port on an ip address.

``ip_addr_id``
    The numeric id of an ipaddr struct.
``ip_addr``
    The actual ipaddr.
``port``
    The port number.
``status``
    The status of the port, either ``open`` or ``closed``.
``banner``
    The service banner we discovered on this port.
``service``
    The service that is running on this port.
``version``
    The version of the service running on this port.

Netblocks
---------

A netblock is a network address range that has been allocated to an individual,
organization or company. Those are commonly found when running whois lookups on
an ip address.

Consider the following example: Running a whois lookup on ``140.82.118.4`` (one
of the addresses currently in use by github) returns that this address belongs
to the netrange ``140.82.112.0 - 140.82.127.255``, so the netblock in this case
is ``140.82.112.0/20``.

``family``
    This is either ``4`` or ``6`` and populated automatically.
``value``
    This is the network range in CIDR notation.
``asn``
    The number of the autonomous system this network belongs to.
``as_org``
    The organization of the autonomous system this network belongs to.
``description``
    This field isn't strictly defined and meant to be used as a human
    meaningful name if available.

CryptoAddrs
-----------

A cryptoaddr is any cryptocurrency address and not tied to a specific currency.

``value``
    The address string. This looks like ``1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2``.
``currency``
    The identifier for a specific currency. This is usually the ticker symbols,
    like ``xbt``, ``zec`` or ``xmr``.
``denominator``
    Balance is tracked internally using 64 bit integers (signed, for technical reasons). Balance is supposed to be the lowest unit, so in case of bitcoin you'd write ``100,000,000`` satoshi instead of ``1`` bitcoin. Since this value is inconvinient to work with we're using the denominator to display values. In case of bitcoin you'd set it to ``8``.
``balance``
    The current balance of the address, in the lowest possible unit. In case of bitcoin this would be satoshis.
``received``
    The total amount of currency received by this address.
``first_seen``
    The first time currency was sent to this address.
``last_withdrawal``
    The last time a transaction signed by this address was observed.
``description``
    A human readable note for this address.

Activity
--------

Activity is different from all other structs, have a look at the `Activity
Section <activity.html>`_.

Relations
---------

Relations are linking two structs together. The link may contain additional information.

subdomain_ipaddr
~~~~~~~~~~~~~~~~

Links an ip address to a subdomain.

``subdomain_id``
    The numeric id of a subdomain struct.
``ip_addr_id``
    The numeric id of an ip addr struct.

network_device
~~~~~~~~~~~~~~

Links a device to a network. This is commonly used with ``db_add_ttl`` so the
link automatically expires. This is frequently used to monitor networks for
known and unknown devices.

``network_id``
    The numeric id of a network struct.
``device_id``
    The numeric id of a device struct.
``ipaddr``
    The ip address assigned to the device.
``last_seen``
    The last time we've seen the device on that network.

breach_email
~~~~~~~~~~~~

Links an email to a breach. If we know the password as well we can add it to
the link. If we don't know the password we can leave it blank and fill it
later. An email can be linked to a breach multiple times with different
passwords. There is a special upserting logic in place to support this.

``breach_id``
    The numeric id of a breach struct.
``email_id``
    The numeric id of an email struct.
``password``
    The password for that email in the breach.
