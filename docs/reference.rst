Function reference
==================

asn_lookup
----------

Run an ASN lookup for a given ip address. The function returns ``asn`` and
``as_org``. This function may fail.

.. code-block:: lua

    lookup = asn_lookup('1.1.1.1')
    if last_err() then return end

base64_decode
-------------

Decode a base64 string with the default alphabet+padding.

.. code-block:: lua

    base64_decode("ww==")

base64_encode
-------------

Encode a binary array with base64 and the default alphabet+padding.

.. code-block:: lua

    base64_encode("\x00\xff")

base64_custom_decode
--------------------

Decode a base64 string with custom alphabet+padding.

.. code-block:: lua

    -- base64
    base64_custom_decode('b2hhaQ==', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '=')
    -- base64 no padding
    base64_custom_decode('b2hhaQ', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '')
    -- base64 url safe
    base64_custom_decode('b2hhaQ==', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_', '=')

base64_custom_encode
--------------------

Encode a binary array with base64 and custom alphabet+padding.

.. code-block:: lua

    -- base64
    base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '=')
    -- base64 no padding
    base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', '')
    -- base64 url safe
    base64_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_', '=')

base32_custom_decode
--------------------

Decode a base32 string with custom alphabet+padding.

.. code-block:: lua

    -- rfc-4648 base32
    base32_custom_decode('N5UGC2I=', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567', '=')
    -- z-base-32
    base32_custom_decode('p7wgn4e', 'ybndrfg8ejkmcpqxot1uwisza345h769', '')

base32_custom_encode
--------------------

Encode a binary array with base32 and custom alphabet+padding.

.. code-block:: lua

    -- rfc-4648 base32
    x = base32_custom_encode('ohai', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567', '=')
    -- z-base-32
    x = base32_custom_encode('ohai', 'ybndrfg8ejkmcpqxot1uwisza345h769', '')

clear_err
---------

Clear the last recorded error from the internal state. See also last_err_.

.. code-block:: lua

    if last_err() then
        -- ignore this error
        clear_err()
    end

create_blob
-----------

Push a byte array into persistent blob storage. This allows passing those bytes
to functions operating on blob storage. Returns a blob identifier that is
deterministic based on the blob content. Blobs are immutable.

.. code-block:: lua

    blob = create_blob("some bytes")
    debug(blob)

datetime
--------

Return current time in UTC. This function is suitable to determine datetimes
for ``DATETIME`` database fields.

.. code-block:: lua

    now = datetime()

.. note::
    This format is sn0int specific, to get the current time for scripting use
    time_unix_ instead.

.. warning::
    This function is going to be deprecated at some point. Prefer sn0int_time_
    for new scripts.

db_add
------

Add an entity to the database or update it if it already exists. This function
may fail or return ``nil``. See `db_add <database.html#db-add>`__ for details.

.. code-block:: lua

    domain_id = db_add('domain', {
        value='example.com',
    })

db_add_ttl
----------

Add a temporary entity to the database. This is commonly used to insert
temporary links that automatically expire over time. If the entity already
exists and is also marked as temporary the new ttl is going to replace the old
ttl. If the entity already exists but never expires we are not going to add a
ttl.

.. code-block:: lua

    -- this link is valid for 2min
    domain_id = db_add_ttl('network-device', {
        network_id=1,
        device_id=13,
    }, 120)

db_activity
-----------

Log an activity event. A basic event looks like this:

.. code-block:: lua

    db_activity({
        topic='harness/activity-ping:dummy',
        time=sn0int_time(),
        content={
            a='b',
            foo={
                bar=1337,
            },
            msg='ohai',
        },
    })

This function is explained in detail in the `activity <activity.html>`_
section.

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

    records = dns('example.com', {
        record='A',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']

.. note::
   DNS replies with an error code set are not causing a change to
   ``last_err()``. You have to test for this explicitly.

.. note::
   This function is unavailable if a socks5 proxy is configured.

error
-----

Log an error to the terminal.

.. code-block:: lua

    error('ohai')

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

hex
---

Hex encode a list of bytes.

.. code-block:: lua

    hex("\x6F\x68\x61\x69\x0A\x00")

hmac_md5
--------

Calculate an hmac with md5. Returns a binary array.

.. code-block:: lua

    hmac_md5("secret", "my authenticated message")

hmac_sha1
---------

Calculate an hmac with sha1. Returns a binary array.

.. code-block:: lua

    hmac_sha1("secret", "my authenticated message")

hmac_sha2_256
-------------

Calculate an hmac with sha2_256. Returns a binary array.

.. code-block:: lua

    hmac_sha2_256("secret", "my authenticated message")

hmac_sha2_512
-------------

Calculate an hmac with sha2_512. Returns a binary array.

.. code-block:: lua

    hmac_sha2_512("secret", "my authenticated message")

hmac_sha3_256
-------------

Calculate an hmac with sha3_256. Returns a binary array.

.. code-block:: lua

    hmac_sha3_256("secret", "my authenticated message")

hmac_sha3_512
-------------

Calculate an hmac with sha3_512. Returns a binary array.

.. code-block:: lua

    hmac_sha3_512("secret", "my authenticated message")

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
``follow_redirects``
  Automatically follow redirects, up to the specified number. If set to 1, only
  one redirect is going to be followed. Defaults to 0 so redirects aren't
  followed.
``body``
  The raw request body as string.
``into_blob``
  If true, the response body is stored in blob storage and a blob reference is
  returned as ``blob`` instead of the full body.
``proxy``
  Use a socks5 proxy in the format ``127.0.0.1:9050``. This option only works
  if it doesn't conflict with the global proxy settings.
``binary``
  Set to ``true`` to get the http response as raw bytes.

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
    if resp['status'] ~= 200 then return 'http status error: ' .. resp['status'] end

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
``binary``
  The response body as bytes (if ``binary=true``)
``blob``
  If ``into_blob`` was enabled for the request the body is downloaded into blob
  storage with a reference to the body in this field.

.. code-block:: lua

    req = http_request(session, 'POST', 'https://httpbin.org/post', {
        json={
            user=user,
            password=password,
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http status error: ' .. resp['status'] end

http_fetch
----------

This does an http_send_ and also automatically validate the status code.

.. note::
   You almost always want this when setting the ``into_blob`` option since this
   function validates the status code *before* inserting the response body into
   blob storage.

.. code-block:: lua

    -- short form
    data = http_fetch(req)
    if last_err() then return end

    -- long form
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http status error: ' .. resp['status'] end

http_fetch_json
---------------

Identical to http_fetch_ but also automatically parses the response body as json.

.. code-block:: lua

    -- short form
    data = http_fetch_json(req)
    if last_err() then return end

    -- long form
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http status error: ' .. resp['status'] end
    data = json_decode(resp['text'])
    if last_err() then return end

img_load
--------

Attempt to decode a blob as an image and return some basic metadata like the
mime type, height and width.

.. code-block:: lua

    img = img_load(blob)
    if last_err() then return end
    debug(img)

img_exif
--------

Extract exif metadata from an image.

.. code-block:: lua

    exif = img_exif(blob)
    if last_err() then return end
    debug(exif)

img_ahash
---------

Calculate the Mean (aHash) perceptual hash.

.. code-block:: lua

    hash = img_ahash(blob)
    if last_err() then return end
    debug(hash)

img_dhash
---------

Calculate the Gradient (dHash) perceptual hash.

.. code-block:: lua

    hash = img_dhash(blob)
    if last_err() then return end
    debug(hash)

img_phash
---------

Calculate the DCT (pHash) perceptual hash.

.. code-block:: lua

    hash = img_phash(blob)
    if last_err() then return end
    debug(hash)

img_nudity
----------

Classify an image for nudity. The score goes from 0 to 2. A score above 1 means
nudity has been detected.

.. code-block:: lua

    nudity = img_nudity(blob)
    if last_err() then return end
    debug(nudity)

info
----

Log an info to the terminal.

.. code-block:: lua

    info('ohai')

intval
------

Parse a number from a string.

    x = strval('1234')

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

key_trunc_pad
-------------

Truncate/pad a key to a given length.

.. code-block:: lua

    -- if longer than 32 bytes: truncate to 32
    -- if shorter than 32 bytes: pad with \x00
    local key = key_trunc_pad(password, 32, 0)

keyring
-------

Request all keys from a given namespace. See the `keyring <keyring.html>`__
section for details.

.. code-block:: lua

    creds = keyring('aws')
    print(creds[1]['accesskey'])
    print(creds[1]['secretkey'])

last_err
--------

Returns infos about the last error we've observed, if any. Returns ``nil``
otherwise.

.. code-block:: lua

    if last_err() then
        -- Something went wrong, abort
        return
    end

md5
---

Hash a byte array with md5 and return the results as bytes.

.. code-block:: lua

    hex(md5("\x00\xff"))

mqtt_connect
------------

Connect to an mqtt broker.

.. code-block:: lua

    local sock = mqtt_connect('mqtts://mqtt.example.com', {
        username='foo',
        password='secret',
    })
    if last_err() then return end

mqtt_subscribe
--------------

Subscribe to a topic. Right now only QoS 0 is supported.

.. code-block:: lua

    mqtt_subscribe(sock, '#', 0)
    if last_err() then return end

mqtt_recv
---------

Receive an mqtt packet. This is not necessarily a publish packet and more
packets might be added in the future, so you need to check the type
specifically.

If a read timeout has been set with mqtt_connect_ this function returns ``nil``
in case of a read timeout.

.. code-block:: lua

    local pkt = mqtt_recv(sock)
    if last_err() then return end
    if pkt == nil then
        -- read timeout, consider sending a ping or disconnect if the previous ping failed
    elseif pkt['type'] == 'pong' then
        -- broker sent a pong
    elseif pkt['type'] == 'publish' then
        local payload = utf8_decode(pkt['body'])
        if last_err() then return end
        info(payload)
    end

mqtt_ping
---------

Send a pingreq packet, causing the broker to send a pingresp. This is used to
make sure the connection is still working correctly.

.. code-block:: lua

    mqtt_ping(sock)
    if last_err() then return end

pgp_pubkey
----------

Same as pgp_pubkey_armored_, but without the unarmor step.

pgp_pubkey_armored
------------------

Extract ``uids``, ``sigs`` and the ``fingerprint`` out of an rfc 4880 pgp
public key. This function may fail.

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
``www.a.b.c.d.example.co.uk`` this is going to be ``example.co.uk``.

.. code-block:: lua

    domain = psl_domain_from_dns_name('www.a.b.c.d.example.co.uk')
    print(domain == 'example.co.uk')

ratelimit_throttle
------------------

Create a ratelimit that can only be passed x times every y milliseconds. This
limit is global for a single ``run`` and also works with threads.

.. code-block:: lua

    -- allow this to pass every 250ms
    ratelimit_throttle('foo', 1, 250)
    -- allow this to pass not more than 4 times per second
    ratelimit_throttle('foo', 4, 1000)

This is useful if you need to coordinate your executions to stay below a
certain request threshold.

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

semver_match
------------

Compare a version to a version requirement. This can be used with
sn0int_version_ to test for certain features or behavior.

.. code-block:: lua

    semver_match('=0.11.2', sn0int_version())
    semver_match('>0.11.2', sn0int_version())
    semver_match('<0.11.2', sn0int_version())
    semver_match('~0.11.2', sn0int_version())
    semver_match('^0.11.2', sn0int_version())
    semver_match('0.11.2', sn0int_version()) -- synonym for ^0.11.2
    semver_match('<=0.11.2', sn0int_version())
    semver_match('>=0.11.2', sn0int_version())
    semver_match('>=0.4.0, <=0.10.0', sn0int_version())

set_err
-------

Manipulate the global error object. If you want to exit the main ``run``
function with an error you can simply return a string, but those are difficult
to propagate through functions. ``set_err`` specifically assigns an error to
the global error object that are also used by all other rust functions.

.. code-block:: lua

    function foo()
        set_err("something failed")
    end

    foo()
    if last_err() then return end

sha1
----

Hash a byte array with sha1 and return the results as bytes.

.. code-block:: lua

    hex(sha1("\x00\xff"))

sha2_256
--------

Hash a byte array with sha2_256 and return the results as bytes.

.. code-block:: lua

    hex(sha2_256("\x00\xff"))

sha2_512
--------

Hash a byte array with sha2_512 and return the results as bytes.

.. code-block:: lua

    hex(sha2_512("\x00\xff"))

sha3_256
--------

Hash a byte array with sha3_256 and return the results as bytes.

.. code-block:: lua

    hex(sha3_256("\x00\xff"))

sha3_512
--------

Hash a byte array with sha3_512 and return the results as bytes.

.. code-block:: lua

    hex(sha3_512("\x00\xff"))

sleep
-----

Pause the current program for the specified number of seconds. This is usually
only used for debugging.

.. code-block:: lua

    sleep(1)

sn0int_time
-----------

Return current time in UTC. This function is suitable to determine datetimes
for ``DATETIME`` database fields.

.. code-block:: lua

    now = sn0int_time()

.. note::
    This format is sn0int specific, to get the current time for scripting use
    time_unix_ instead.

sn0int_time_from
----------------

Identical to sn0int_time_ but uses a unix timestamp in seconds instead of the
current time. This function is compatible with time_unix_ and strptime_.

.. code-block:: lua

    time = sn0int_time_from(1567931337)

sn0int_version
--------------

Get the current sn0int version string. This can be used with semver_match_ to
test for certain features or behavior.

.. code-block:: lua

    info(sn0int_version())

sock_connect
------------

Create a tcp connection.

The following options are available:

``tls``
  Set to true to enable tls (certificates are validated)
``sni_value``
  Instead of the host argument, use a custom string for the sni extension.
``disable_tls_verify``
  **Danger**: disable tls verification. This disables all security on the
  connection. Note that sn0int is still rather strict, you're going to run into
  issues if you need support for insecure ciphers.
``proxy``
  Use a socks5 proxy in the format ``127.0.0.1:9050``. This option only works
  if it doesn't conflict with the global proxy settings.
``connect_timeout``
  Abort tcp connection attempts after ``n`` seconds.
``read_timeout``
  Abort read attempts after ``n`` seconds. This can be used to wake up
  connections periodically.
``write_timeout``
  Abort write attempts after ``n`` seconds.

.. code-block:: lua

    sock = sock_connect("127.0.0.1", 1337, {
        tls=true,
    })

sock_upgrade_tls
----------------

Take an existing tcp connection and start a tls handshake. The options are the
same as sock_connect_ but the ``tls`` value is always assumed to be true.

The sni value needs to be set specifically, otherwise the sni extension is
disabled.

Using this function specifically returns some extra information that is
discarded when using sock_connect_ directly with ``tls=true``.

.. code-block:: lua

    sock = sock_connect("127.0.0.1", 1337, {})
    if last_err() then return end

    tls = sock_upgrade_tls(sock, {
        sni_value='example.com',
    })
    if last_err() then return end

    info(tls)

sock_options
------------

Update options of an existing connection:

``read_timeout``
  Abort read attempts after ``n`` seconds. This can be used to wake up
  connections periodically.
``write_timeout``
  Abort write attempts after ``n`` seconds.

.. code-block:: lua

    sock_options(sock, {
        read_timeout=3,
    })

sock_send
---------

Send data to the socket.

.. code-block:: lua

    sock_send(sock, "hello world")

sock_recv
---------

Receive up to 4096 bytes from the socket.

.. code-block:: lua

    x = sock_recv(sock)

sock_sendline
-------------

Send a string to the socket. A newline is automatically appended to the string.

.. code-block:: lua

    sock_sendline(sock, line)

sock_recvline
-------------

Receive a line from the socket. The line includes the newline.

.. code-block:: lua

    x = sock_recvline(sock)

sock_recvall
------------

Receive all data from the socket until EOF.

.. code-block:: lua

    x = sock_recvall(sock)

sock_recvline_contains
----------------------

Receive lines from the server until a line contains the needle, then return
this line.

.. code-block:: lua

    x = sock_recvline_contains(sock, needle)

sock_recvline_regex
-------------------

Receive lines from the server until a line matches the regex, then return this
line.

.. code-block:: lua

    x = sock_recvline_regex(sock, "^250 ")

sock_recvn
----------

Receive exactly n bytes from the socket.

.. code-block:: lua

    x = sock_recvn(sock, 4)

sock_recvuntil
--------------

Receive until the needle is found, then return all data including the needle.

.. code-block:: lua

    x = sock_recvuntil(sock, needle)

sock_sendafter
--------------

Receive until the needle is found, then write data to the socket.

.. code-block:: lua

    sock_sendafter(sock, needle, data)

sock_newline
------------

Overwrite the default ``\n`` newline.

.. code-block:: lua

    sock_newline(sock, "\r\n")

sodium_secretbox_open
---------------------

Use authenticated symetric crypto to decrypt a given message.

Internally this is ``crypto_secretbox_xsalsa20poly1305``.

The key **must** be 32 bytes, see key_trunc_pad_ if necessary.

The first 24 bytes of the encrypted message are expected to be the nonce.

.. code-block:: lua

    plain = sodium_secretbox_open(encrypted, key)
    if last_err() then return end

    txt = utf8_decode(plain)
    if last_err() then return end

    info(txt)

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

.. TODO: add stdin_read_line and deprecate stdin_readline

stdin_read_to_end
-----------------

Read stdin until EOF as a utf-8 string.

.. code-block:: lua

    stdin_read_to_end()

.. note::
   This only works with `sn0int run --stdin`.

str_find
--------

Returns the byte index of the first character that matches the pattern. This is
explicitly a literal match instead of a lua pattern.

If no match is found, returns ``nil``.

.. code-block:: lua

    x = str_find('asdf', 'sd')
    print(x == 2)

str_replace
-----------

Replaces all matches of a pattern in a string. This is explicitly a literal
match instead of a lua pattern.

If no match is found, an unmodified copy is returned.

.. code-block:: lua

    x = str_replace('this is old', 'old', 'new')
    print(x == 'this is new')

strftime
--------

Format a timestamp generated with time_unix_ into a date, see `strftime rules`_.

.. code-block:: lua

    t = strftime('%d/%m/%Y %H:%M', 1558584994)

strptime
--------

Parse a date into a unix timestamp, see `strftime rules`_.

.. code-block:: lua

    t = strptime('%d/%m/%Y %H:%M', '23/05/2019 04:16')

.. _strftime rules: https://docs.rs/chrono/0.4.6/chrono/format/strftime/index.html

strval
------

Convert a number into a string.

    x = strval(1234)

time_unix
---------

Get the current time as seconds since ``January 1, 1970 0:00:00 UTC``, also
known as UNIX timestamp. This timestamp can be formated using strftime_.

.. code-block:: lua

    now = time_unix()

url_decode
----------

Parse a query string into a map. For raw percent decoding see url_unescape_.

.. code-block:: lua

    v = url_decode('a=b&c=d')
    print(v['a'] == 'b')
    print(v['c'] == 'd')

url_encode
----------

Encode a map into a query string. For raw percent encoding see url_escape_.

.. code-block:: lua

    v = url_encode({
        a='b',
        c='d',
    })
    print(v == 'a=b&c=d')

url_escape
----------

Apply url escaping to a string.

.. code-block:: lua

    v = url_escape('foo bar?')
    print(v == 'foo%20bar%3F')

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

    url = url_parse('https://example.com')
    print(url['scheme'] == 'https')
    print(url['host'] == 'example.com')
    print(url['path'] == '/')

url_unescape
------------

Remove url escaping of a string.

.. code-block:: lua

    v = url_unescape('foo%20bar%3F')
    print(v == 'foo bar?')

utf8_decode
-----------

Decodes a list of bytes/numbers into a string. This function might fail.

.. code-block:: lua

    x = utf8_decode({65, 65, 65, 65})
    if last_err() then return end
    print(x == 'AAAA')

warn
----

Log a warning to the terminal.

.. code-block:: lua

    warn('ohai')

warn_once
---------

Log a warning to the terminal once. This can be used to print a warning to the
user without printing the same warning for each struct we're processing during
a ``run`` execution.

.. code-block:: lua

    warn_once('ohai')
    warn_once('ohai')

ws_connect
----------

Create a websocket connection. The url format is ``ws://example.com/asdf``,
``wss://`` is also supported.

The following options are available:

``headers``
  A map of additional headers that should be set for the request.
``proxy``
  Use a socks5 proxy in the format ``127.0.0.1:9050``. This option only works
  if it doesn't conflict with the global proxy settings.
``connect_timeout``
  Abort tcp connection attempts after ``n`` seconds.
``read_timeout``
  Abort read attempts after ``n`` seconds. This can be used to wake up
  connections periodically.
``write_timeout``
  Abort write attempts after ``n`` seconds.

.. code-block:: lua

    sock = ws_connect("wss://example.com/asdf", {})

ws_options
----------

Update options of an existing connection:

``read_timeout``
  Abort read attempts after ``n`` seconds. This can be used to wake up
  connections periodically.
``write_timeout``
  Abort write attempts after ``n`` seconds.

.. code-block:: lua

    ws_options(sock, {
        read_timeout=3,
    })

ws_recv_text
------------

Wait until the server sends a text frame. A binary frame is considered an
error. Ping requests are answered automatically.

.. code-block:: lua

    msg = ws_recv_text(sock)

ws_recv_binary
--------------

Wait until the server sends a binary frame. A text frame is considered an
error. Ping requests are answered automatically.

.. code-block:: lua

    msg = ws_recv_binary(sock)

ws_recv_json
------------

Identical to ws_send_text_ but automatically runs json_decode_ on the
response.

.. code-block:: lua

    msg = ws_recv_json(sock)

ws_send_text
------------

Send a text frame on the websocket connection.

.. code-block:: lua

    ws_send_text(sock, "ohai!")

ws_send_binary
--------------

Send a binary frame on the websocket connection.

.. code-block:: lua

    ws_send_binary(sock, "\x00\x01\x02")

ws_send_json
------------

Encode the object as json string and send it as a text frame on the websocket
connection.

.. code-block:: lua

    ws_send_text(sock, {
        foo="ohai!",
        x={
            y={1,3,3,7},
        },
    })

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

xml_decode
----------

Decode a lua value from an xml document.

.. code-block:: lua

    x = xml_decode('<body><foo fizz="buzz">bar</foo></body>')
    if last_err() then return end

    body = x['children'][1]
    foo = body['children'][1]

    print(foo['attrs']['fizz'])
    print(foo['text'])

xml_named
---------

Get a named child element from a parent element.

.. code-block:: lua

    x = xml_decode('<body><foo fizz="buzz">bar</foo></body>')
    if last_err() then return end

    body = x['children'][1]
    foo = xml_named(body, 'foo')
    if foo ~= nil then
        print(foo)
    end
