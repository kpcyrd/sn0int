Activity
========

So far we've learned about regular `structs <structs.html>`_, but activity is
special.

Activity is an event tied to a specific time and topic and has a small amount
of data piggybacked to it.

Anatomy of an event
-------------------

``topic``
    This is some freestyle text used to group events to a specific topic. This
    must not conflict with other modules unless there's a very good reason.

    The topic should look like ``kpcyrd/example:something``, with ``something``
    being a meaningful unique identifier for whatever is generating these
    events, like a mac address or an account name/id.

    The rules around this might become stricter in the future.
``time``
    The most important part of the event: The time and date it happened.
``initial``
    This value can not be set but might be present in sn0int output. See `Querying events`_.
``uniq`` (optional)
    This is an optional feature to deduplicate events. Assuming you're
    importing posts by an account, you wouldn't want to store a new event for
    each post you already imported. If you set this field to the technical post
    id then sn0int would skip the event if it already has an event with the
    same ``topic`` and ``uniq`` combination to avoid inserting duplicates.
``latitude`` (optional)
    Latitude - if you can tie the event to a specific location.
``longitude`` (optional)
    Longitude - if you can tie the event to a specific location.
``radius`` (optional)
    The location radius in meters. If the position you got has a precision of
    100 meters set this value to ``100``.
``content``
    Arbitrary data that you want to attach to the event. This doesn't need to
    be a string and can be an arbitrary object that is then stored as json
    string.

Logging events
--------------

An ``activity`` event can be logged with ``db_activity``:

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

Logging an event that has a location attached could look like this:

.. code-block:: lua

    db_activity({
        topic='harness/activity-ping:dummy',
        time=sn0int_time(),
        latitude=40.726662,
        longitude=-74.036677,
        radius=50,
        content={
            a='b',
            foo={
                bar=1337,
            },
            msg='ohai',
        },
    })

Making sure an event is not logged twice can be done with ``uniq``:

.. code-block:: lua

    -- create the first event
    db_activity({
        topic='harness/activity-ping:dummy',
        time=sn0int_time(),
        uniq='12345',
        content='ohai',
    })

    -- this does nothing because we already have an event with this topic+uniq combination
    db_activity({
        topic='harness/activity-ping:dummy',
        time=sn0int_time(),
        uniq='12345',
        content='ohai',
    })

    -- this creates a new event because uniq is different
    db_activity({
        topic='harness/activity-ping:dummy',
        time=sn0int_time(),
        uniq='6789',
        content='ohai',
    })

    -- this also creates a new event because topic is different
    db_activity({
        topic='harness/activity-ping:something-else',
        time=sn0int_time(),
        uniq='6789',
        content='ohai',
    })

Querying events
---------------

There is a commandline interface that can be used to query all events we've
logged. To get everything (sorted by time)::

    sn0int activity

To limit the output to a specific topic::

    sn0int activity -t harness/activity-ping:dummy

To limit it to a specific time frame::

    # everything since
    sn0int activity --since 2020-01-13T04:20:00
    # everything until
    sn0int activity --until 2020-01-13T04:20:00
    # both
    sn0int activity --since yesterday --until today

When using ``--since`` you might also want to know the previous state and use
it as an initial value. Consider this example::

    2020-01-13 14:30:00 # user goes offline
    2020-01-13 23:59:00 # user goes online
    2020-01-14 09:30:00 # user goes idle
    2020-01-14 14:20:00 # user goes offline

If we're running a query like ``sn0int activity --since 2020-01-14T00:00:00``
the program consuming the output wouldn't know that the user is initially
online because we're only getting this data::

    {"id":8,"topic":"foo/bar:asdf","time":"2020-01-14T09:30:00","content":{"state":"idle"}}
    {"id":9,"topic":"foo/bar:asdf","time":"2020-01-14T14:20:00","content":{"state":"offline"}}

We can tweak this with ``sn0int activity --initial --since
2020-01-14T00:00:00`` to include one more event that we only use to populate
the intial state::

    {"id":7,"initial":true,"topic":"foo/bar:asdf","time":"2020-01-13T23:59:00","content":{"state":"online"}}
    {"id":8,"topic":"foo/bar:asdf","time":"2020-01-14T09:30:00","content":{"state":"idle"}}
    {"id":9,"topic":"foo/bar:asdf","time":"2020-01-14T14:20:00","content":{"state":"offline"}}

Visualization
-------------

There is no visualization built in, there may be external frontends for this in
the future. You're very welcome to write one!
