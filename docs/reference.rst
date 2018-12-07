Function reference
==================

clear_err
---------

Clear the last recorded error from the internal state. See also last_err_.

.. code-block:: lua

    if last_err() then
        -- ignore this error
        clear_err()
    end

db_add
------

Add an entity to the database or update it if it already exists. This function
may fail or return ``nil``. See `db_add <database.html#db-add>`__ for details.

.. code-block:: lua

    domain_id = db_add('domain', {
        value='example.com',
    })

db_select
---------

Checks if a target is in scope. If non-nil is returned, this entity is in
scope. This function may fail. See `db_select <database.html#db-select>`__ for
details.

.. code-block:: lua

    domain_id = db_select('domain', 'example.com')
    if domain_id ~= nil then
        -- do something
    end

db_update
---------

Update an entity in the database. This function may fail. See `db_update
<database.html#db-update>`__ for details.

.. code-block:: lua

    db_update('ipaddr', arg, {
        asn=lookup['asn'],
        as_org=lookup['as_org'],
    })

dns
---

Resolve a dns record. If the dns query was successful and the dns reply is
``NoError`` then ``x['error']`` is ``nil``. The records of the reply are in
``x['answers']``. This function may fail.

This function accepts the following options:

``record``
  The ``query_type``, can be any of ``A``, ``AAAA``, ``MX``, ``AXFR``, etc.
``nameserver``
  The server that should be used for the lookup. Defaults to your system
  resolver.
``tcp``
  If the lookup should use tcp, true/false.
``timeout``
  The time until the query times out in milliseconds.

.. code-block:: lua

    x = dns('example.com', {
        record='A'
    })
    if last_err() then return end

.. note::
   DNS replies with an error code set are not causing a change to
   ``last_err()``. You have to test for this explicitly.

error
-----

Log an error to the terminal.

.. code-block:: lua

    error('ohai')

asn_lookup
----------

Run an ASN lookup for a given ip address. The function returns ``asn`` and
``as_org``. This function may fail.

.. code-block:: lua

    lookup = asn_lookup('1.1.1.1')
    if last_err() then return end

geoip_lookup
------------

Run a geoip lookup for a given ip address. The function returns:

- continent
- continent_code
- country
- country_code
- city
- latitude
- longitude

This function may fail.

.. code-block:: lua

    lookup = geoip_lookup('1.1.1.1')
    if last_err() then return end

html_select
-----------

Parses an html document and returns the first element that matches the css
selector. The return value is a table with `text` being the inner text and
`attrs` being a table of the elements attributes.

.. code-block:: lua

    csrf = html_select(html, 'input[name="csrf"]')
    token = csrf["attrs"]["value"]

html_select_list
----------------

Same as html_select_ but returns all matches instead of the first one.

.. code-block:: lua

    html_select_list(html, 'input[name="csrf"]')

http_mksession
--------------

Create a session object. This is similar to ``requests.Session`` in
python-requests and keeps track of cookies.

.. code-block:: lua

    session = http_mksession()

http_request
------------

Prepares an http request. The first argument is the session reference and
cookies from that session are copied into the request. After the request has
been sent, the cookies from the response are copied back into the session.

The next arguments are the ``method``, the ``url`` and additional options.
Please note that you still need to specify an empty table ``{}`` even if no
options are set. The following options are available:

``query``
  A map of query parameters that should be set on the url.
``headers``
  A map of headers that should be set.
``basic_auth``
  Configure the basic auth header with ``{"user, "password"}``.
``user_agent``
  Overwrite the default user agent with a string.
``json``
  The request body that should be json encoded.
``form``
  The request body that should be form encoded.
``body``
  The raw request body as string.

This function may fail.

.. code-block:: lua

    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp["status"] ~= 200 then return "invalid status code" end

http_send
---------

Send the request that has been built with http_request_. Returns a table with
the following keys:

``status``
  The http status code
``headers``
  A table of headers
``text``
  The response body as string

.. code-block:: lua

    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp["status"] ~= 200 then return "invalid status code" end

info
----

Log an info to the terminal.

.. code-block:: lua

    info('ohai')

json_decode
-----------

Decode a lua value from a json string.

.. code-block:: lua

    json_decode("{\"data\":{\"password\":\"fizz\",\"user\":\"bar\"},\"list\":[1,3,3,7]}")

json_decode_stream
------------------

Very similar to json_decode_, but works with multiple json objects directly
concatenated to each other or separated by newlines.

.. code-block:: lua

    json_decode_stream("{\"data\":1}{\"data\":2}")

json_encode
-----------

Encode a datastructure into a string.

.. code-block:: lua

    x = json_encode({
        some=1,
        fancy={
            data='structures',
        }
    })
    print(x)

last_err
--------

Returns infos about the last error we've observed, if any. Returns ``nil`` otherwise.

.. code-block:: lua

    if last_err() then
        -- Something went wrong, abort
        return
    end

pgp_pubkey
----------

Same as pgp_pubkey_armored_, but without the unarmor step.

pgp_pubkey_armored
------------------

Extract uids out of a rfc 4880 pgp public key. This function may fail.

.. code-block:: lua

    key = pgp_pubkey_armored([===[
    -----BEGIN PGP PUBLIC KEY BLOCK-----
    Version: GnuPG v2

    mQENBFu6q90BCADgD7Q9aH5683yt7hzPktDkAUNAZJHwYhUNeyGK43frPyDRWQmq
    N+oXTfiYWLQN+d7KNBTnF9uwyBdaLM7SH44lLNYo8W09mVM2eK+wt19uf5HYNgAE
    8la45QLo/ce9CQVe1a4oXNWq6l0FOY7M+wLe+G2wMwz8RXGgwd/qQp4/PB5YpUhx
    nAnzClxvwymrL6BQXsRcKSMSD5bIzIv95n105CvW5Hql7JR9zgOR+gHqVOH8HBUc
    ZxMumrTM6aKLgAhgM8Sn36gCFOfjlG1b1OFLZhUtgro/nnEOmAurRsCZy8M5h8QM
    FpZChIH8kgHs90F/CCvGjMq3qvWcH8ZsPUizABEBAAG0NUhhbnMgQWNrZXIgKGV4
    YW1wbGUgY29tbWVudCkgPGhhbnMuYWNrZXJAZXhhbXBsZS5jb20+iQFOBBMBCAA4
    FiEEyzeO1eEwbB03hcqBM00IodGdlj8FAlu6q90CGwMFCwkIBwIGFQgJCgsCBBYC
    AwECHgECF4AACgkQM00IodGdlj/AJQgAjmk+iP5b7Jt7+f+lU4Oprlf3f3DG/uh5
    Ge6MjV7cvtxlhZJRD5hxGt9RwwnEp61TBSbrem288pM89ilQfTNe0wUr9OzwWzh/
    8Ngl5iWnD2ah3Mpi5R1V/YMNf2cnwVjqNvfkRHdNc43pZOkC2GoiTUn0QY0UBpOW
    ZMN3//ANi6ZtiK/L0IZQND/gKvOzu/4tfaJeBl26T3cVYj53p3G3jhlb92vVa8SR
    uL3S3bzd1h5snDgU1uXHmNHGbhkEc4KUneQ0V9/bdZrg6OzFAfM1ghgfoId+YpQH
    er9L26ISL3QF58wdEXfIdHYEmMlANjBMO2cUlQXgONuCgkMuY7GBmrkBDQRbuqvd
    AQgA41jqCumCxYV0NdSYNnTSSDRyd69dOUYCAPT80iZ739s7KKJS9X9KVfGmDjfi
    u2RcfR/KYj53HoyOm4Pm/+ONN8De4ktzXpIpJxGC+O8NBvd9vkboAS6qnCjK7KVE
    r91ymxxVKp2dzZvVfpIjWVZR5i2EAvS5vw8UK4gL8ALH+S9leJFZrQWcgyoJOJzH
    Rzr9pesX2HvdgcNG1O6QUArlsnsTnqpi/hu7tQa8tifBpWDeArOA23Y2DgeehdDF
    lSU/8KD4J+AkFrWWlcTaMsvSChXQkCHEMRIcSOfXtdpX5KJSE7UBQdD1opm+mR79
    VeHnuJAAVZZtUZmJA7pjdKykYQARAQABiQE2BBgBCAAgFiEEyzeO1eEwbB03hcqB
    M00IodGdlj8FAlu6q90CGwwACgkQM00IodGdlj8bMAf+Lq3Qive4vcrCTT4IgvVj
    arOACdcbtt5RhVBTimT19rDWNH+m+PfPjo3FSlBj5cm70KAXUS2LBFFxhakTZ/Mq
    cQroWZpVbBxj4kipEVVJZFdUZQaDERJql0xYGOQrNMQ4JGqJ84BRrtOExjSqo41K
    hAhNe+bwPGH9/Igiixc4tH07xa7TOy4MyJv/6gpbHy/lW1hqpCAgM5fT/im5/6QF
    k0tED6vIuc54IWiOmwCnjZiQnJ8uCwEu+cuJ5Exwy9CNERLp5v0y4eG+0E+at9j/
    macOg39qf09t53pTqe9dWv5NIi319TeBsKZ2lb0crrQjsbHqk0DAUwgQuoANqLku
    vA==
    =kRIv
    -----END PGP PUBLIC KEY BLOCK-----
    ]===])

    if last_err() then return end
    print(key)

print
-----

Write something directly to the terminal.

.. code-block:: lua

    print({
        some=1,
        fancy={
            data='structures',
        }
    })

.. warning::
   This function writes directly to the terminal and can interfere with other
   terminal features. This function should be used during development only.

psl_domain_from_dns_name
------------------------

Returns the parent domain according to the public suffix list. For
``www.a.b.c.d.example.com`` this is going to be ``example.com``.

.. code-block:: lua

    domain = psl_domain_from_dns_name('www.a.b.c.d.example.com')
    print(domain == 'example.com')

regex_find
----------

Apply a regex to some text. Returns ``nil`` if the regex didn't match and the
capture groups if it did.

.. code-block:: lua

    m = regex_find(".(.)", "abcdef")

    if m == nil then
        print('No captures')
    end

    print(m[1] == 'ab')
    print(m[2] == 'b')

regex_find_all
--------------

Same as regex_find_, but returns all matches.

.. code-block:: lua

    m = regex_find_all(".(.)", "abcdef")

    print(m[1][1] == 'ab')
    print(m[1][2] == 'b')
    print(m[2][1] == 'cd')
    print(m[2][2] == 'd')
    print(m[3][1] == 'ef')
    print(m[3][2] == 'f')

sleep
-----

Pause the current program for the specified number of seconds. This is usually
only used for debugging.

.. code-block:: lua

    sleep(1)

status
------

Update the label of the progress indicator.

.. code-block:: lua

    status('ohai')

stdin_readline
--------------

Read a line from stdin. The final newline is not removed.

.. code-block:: lua

    stdin_readline()

.. note::
   This only works with `sn0int run --stdin`.

url_join
--------

Join a relative link to an absolute link. If both links are absolute we just
return the first one:

.. code-block:: lua

    x = url_join('https://example.com/x', '/foo')
    print(x == 'https://example.com/foo')

    x = url_join('https://example.com/x', 'https://github.com/')
    print(x == 'https://github.com/')

url_parse
---------

Parse a url into its components. The following components are returned:

- scheme
- host
- port
- path
- query
- fragment
- params

.. code-block:: lua

    url = url_parse("https://example.com")
    print(url['scheme'] == 'https')
    print(url['host'] == 'example.com')
    print(url['path'] == '/')

utf8_decode
-----------

Decodes a list of bytes/numbers into a string. This function might fail.

.. code-block:: lua

    x = utf8_decode({65, 65, 65, 65})
    if last_err() then return end
    print(x == 'AAAA')

x509_parse_pem
--------------

Parse a pem encoded certificate. This function might fail.

.. code-block:: lua

    x = x509_parse_pem([[-----BEGIN CERTIFICATE-----
    MIID9DCCA3qgAwIBAgIQBWzetBRl/ycHFsBukRYuGTAKBggqhkjOPQQDAjBMMQsw
    CQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMSYwJAYDVQQDEx1EaWdp
    Q2VydCBFQ0MgU2VjdXJlIFNlcnZlciBDQTAeFw0xODAzMzAwMDAwMDBaFw0yMDAz
    MjUxMjAwMDBaMGwxCzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMN
    U2FuIEZyYW5jaXNjbzEZMBcGA1UEChMQQ2xvdWRmbGFyZSwgSW5jLjEdMBsGA1UE
    AwwUKi5jbG91ZGZsYXJlLWRucy5jb20wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNC
    AASyRQsxrFBjziHmfDQjGsXBU0WWl3oxh7vg6h2V9f8lBMp18PY/td9R6VvJPa20
    AwVzIJI+dL6OSxviaIZEbmK7o4ICHDCCAhgwHwYDVR0jBBgwFoAUo53mH/naOU/A
    buiRy5Wl2jHiCp8wHQYDVR0OBBYEFN+XTeVDs7BBp0LykM+Jf64SV4ThMGMGA1Ud
    EQRcMFqCFCouY2xvdWRmbGFyZS1kbnMuY29thwQBAQEBhwQBAAABghJjbG91ZGZs
    YXJlLWRucy5jb22HECYGRwBHAAAAAAAAAAAAERGHECYGRwBHAAAAAAAAAAAAEAEw
    DgYDVR0PAQH/BAQDAgeAMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcDAjBp
    BgNVHR8EYjBgMC6gLKAqhihodHRwOi8vY3JsMy5kaWdpY2VydC5jb20vc3NjYS1l
    Y2MtZzEuY3JsMC6gLKAqhihodHRwOi8vY3JsNC5kaWdpY2VydC5jb20vc3NjYS1l
    Y2MtZzEuY3JsMEwGA1UdIARFMEMwNwYJYIZIAYb9bAEBMCowKAYIKwYBBQUHAgEW
    HGh0dHBzOi8vd3d3LmRpZ2ljZXJ0LmNvbS9DUFMwCAYGZ4EMAQICMHsGCCsGAQUF
    BwEBBG8wbTAkBggrBgEFBQcwAYYYaHR0cDovL29jc3AuZGlnaWNlcnQuY29tMEUG
    CCsGAQUFBzAChjlodHRwOi8vY2FjZXJ0cy5kaWdpY2VydC5jb20vRGlnaUNlcnRF
    Q0NTZWN1cmVTZXJ2ZXJDQS5jcnQwDAYDVR0TAQH/BAIwADAKBggqhkjOPQQDAgNo
    ADBlAjEAjoyy2Ogh1i1/Kh9+psMc1OChlQIvQF6AkojZS8yliar6m8q5nqC3qe0h
    HR0fExwLAjAueWRnHX4QJ9loqMhsPk3NB0Cs0mStsNDNG6/DpCYw7XmjoG3y1LS7
    ZkZZmqNn2Q8=
    -----END CERTIFICATE-----
    ]])
    if last_err() then return end
    print(x)
