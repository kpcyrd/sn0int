Keyring
=======

A common problem is that you need either an api key or a username/password
combination. Instead of hardcoding it in the script you should request them
from the keyring. In order to do this you need to request permissions to those
credentials.

Managing the keyring
--------------------

The keyring is a simple namespaced key-value store::

    [sn0int][default] > keyring add aws:AKIAIOSFODNN7EXAMPLE
    Secretkey: keep-this-secret
    [sn0int][default] > keyring list
    aws:AKIAIOSFODNN7EXAMPLE
    [sn0int][default] >
    [sn0int][default] > keyring list aws
    aws:AKIAIOSFODNN7EXAMPLE
    [sn0int][default] > keyring list instagram
    [sn0int][default] >
    [sn0int][default] > keyring get aws:AKIAIOSFODNN7EXAMPLE
    Namespace:    "aws"
    Access Key:   "AKIAIOSFODNN7EXAMPLE"
    Secret:       "keep-this-secret"
    [sn0int][default] >

If the service uses a username-password combination, set the username as the
access key and the password as the secret.

If the service uses only a secret key for the api, set the secret key as the
access key and leave the secret blank.

A script doesn't automatically get access to requested keyring namespaces.
Instead the user is asked to confirm those requests to limit abusive scripts.

Using access keys in scripts
----------------------------

We can request all keys of a certain namespace in our script metadata. This is
going to prompt the user to grant the script access. This can be done for
multiple namespaces in the same script:

.. code-block:: lua

    -- Keyring-Access: aws
    -- Keyring-Access: asdf

If the user granted us access to those keys we can read them with ``keyring``:

.. code-block:: lua

    creds = keyring('aws')
    debug(creds[1]['access_key'])
    debug(creds[1]['secret_key'])

This returns a list of all keys in that namespace. Any empty list is returned
if the user doesn't have any keys in that namespace.

If you want to allow the user to select a specific script you can introduce an
option that is set by the user and then filter ``creds`` until the
``access_key`` matches.

Using access keys as source argument
------------------------------------

We can also use the access keys as source argument. This is useful if each
account has access to different things and we want to read through all of them.

Since access key permissions are granted per namespace we need to specify which
credentials we want to use.

.. code-block:: lua

    -- Keyring-Access: aws
    -- Source: keyring:aws
