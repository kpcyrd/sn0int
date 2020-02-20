Database
========

There are a few things you need to understand how the database works to use it
efficiently.

The database that is backing sn0int is sqlite, but the api that is exposed to
the user and scripts is an nosql-ish object store. The query language that is
exposed to the user is still very similar to sql, except that it lacks a column
statement::

    select subdomains where value like %.example.com
    ^      ^          ^           ^    ^
    |      |          |           |    this value is going to be quoted automatically
    |      |          |           |
    |      |          |           this triggers automatic quoting
    |      |          |
    |      |          apply a filter, this translates to sql quite literally
    |      |
    |      the entity we want to select is a subdomain.
    |      this affects the table and the deserializer
    |
    select entities

This is how almost all user facing functions work that operate on the database.
The functions that are available for scripting are a bit more object based and
described below.

db_add
------

This operation is somewhat straight forward. It adds an entity to the
database:

.. code-block:: lua

    domain_id = db_add('domain', {
        value='example.com',
    })

If this entity conflicts with an entity that already exists, an upsert is
triggered and an db_update is performed instead.

.. note::
   This function may return ``nil`` if the entity already exists, but has been
   removed from scope with ``noscope``. Everytime you use ``db_add`` you need
   to make sure that the ID that has been returned is not ``nil``.

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

db_update
---------

Update some mutable fields of an entity:

.. code-block:: lua

    db_update('ipaddr', arg, {
        asn=lookup['asn'],
        as_org=lookup['as_org'],
    })

The first parameter is usually the same arg that your script was called with.
Usually you can use db_add instead of db_update due to the upsert feature, but
db_update is still slightly faster.

.. note::
   Some fields are immutable and can not be updated.

db_select
---------

This function is used to check if something is in scope. If the entity has been
added to the database and has not been removed from scope, this function
returns that entities id. This is somewhat similar to ``db_add``, except that
``db_select`` never adds anything to the database.

.. code-block:: lua

    domain_id = db_select('domain', 'example.com')
    if domain_id ~= nil then
        -- do something
    end

This function only accepts a string instead of a lua table. This string is used
to filter on the ``value`` column.
