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

TODO: permission system

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
    print(creds[1]['accesskey'])
    print(creds[1]['secretkey'])

This returns an empty list if the user doesn't have any keys in that namespace.

.. note::
   The argument for ``keyring`` is supposed to be a namespace name which
   returns all secrets in that namespace, but you can also select a specific
   key by specifying the full ``namespace:name`` combination.

Using access keys as source argument
------------------------------------

TODO: this is currently not supported

We can also use the access keys as source argument. This is useful if each
account has access to different things and we want to read through all of them.

Since access key permissions are granted per namespace we need to specify which
credentials we want to use.

.. code-block:: lua

    -- Keyring-Access: aws
    -- Source: keyring:aws
