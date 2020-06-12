Notifications
=============

If you run sn0int unattended nobody might see the sn0int output. For cases like
this you can configure notifications to send you a push notification in case
something interesting happens. This is also especially useful if you have
sn0int setup to run automatically.

Receiving notifications
-----------------------

Notifications are just regular sn0int modules. You can install them just like
any other module or write your own. This section contains walkthroughs on how
to setup common integrations.

Telegram
~~~~~~~~

Install the telegram notification module from the registry:

.. code-block:: bash

    sn0int pkg install kpcyrd/notify-telegram

Open your telegram app and open a chat with ``@botfather``. Send ``/newbot``
and answer the questions. Copy ``bot_token`` and open this url in your browser:

.. code-block::

    https://api.telegram.org/bot**your_bot_token**/getUpdates

Back on your app, open the t.me link to start a new chat with your bot, then
send ``/start``. Reload the page in your browser, you should see the new
message you sent. Copy the ``chat_id``.

Test your tokens are working correctly by sending yourself a notification:

.. code-block:: bash

    sn0int notify exec kpcyrd/notify-telegram -o bot_token=1337:foobar -o chat_id=1337 'hello world'

You should receive ``hello world`` from your bot on Telegram.

Pushover
~~~~~~~~

Install the pushover notification module from the registry:

.. code-block:: bash

    sn0int pkg install kpcyrd/notify-pushover

Signup for pushover and configure the app on your device. Copy th user key
visible on the pushover dashboard. Click "Create an Application/API Token". Set
"sn0int" as name and set an icon if you want to. Copy the api token.

Test your tokens are working correctly by sending yourself a notification:

.. code-block:: bash

    sn0int notify exec kpcyrd/notify-pushover -o user_key=asdf1337 -o api_token=asdf1337 'hello world'

You should receive ``hello world`` as a push notification.

Discord
~~~~~~~

Install the discord notification module from the registry:

.. code-block:: bash

    sn0int pkg install kpcyrd/notify-discord

Decide which channel should receive notifications (or create a new one). Open
the "Server Settings" of your discord server. Click on "Webhooks". Click
"Create Webhook". Configure the Name and Channel. Copy the Webhook URL.


Test your tokens are working correctly by sending yourself a notification:

.. code-block:: bash

    sn0int notify exec kpcyrd/notify-discord -o url=https://discord.com/api/webhooks/1337/asdf 'hello world'

You should receive ``hello world`` in your discord channel.

Signal
~~~~~~

Install the sn0int notification module from the registry:

.. code-block:: bash

    sn0int pkg install kpcyrd/notify-signal

This module allows end-to-end encrypted notifications, but it's also difficult
to setup. You need a second phone number and install both `signal-cli
<https://github.com/AsamK/signal-cli>`_ and `sn0int-signal
<https://github.com/kpcyrd/sn0int-signal>`_.

After you've registered your second phone number with signal-cli, you can use
sn0int-signal to expose a minimal api for notify-signal. For more detailed
instructions and how to start the api at boot, see the `sn0int-signal README
<https://github.com/kpcyrd/sn0int-signal>`_.

Read the secret key generated at ``/etc/sn0int-signal.key`` and send a
notification to the signal phone number:

.. code-block:: bash

    sn0int notify exec kpcyrd/notify-signal -o to=+31337 -o secret=asdf 'hello world'

You should receive ``hello world`` from the number signed up with signal-cli.

Writing your own module
~~~~~~~~~~~~~~~~~~~~~~~

Make sure you've read the detailed instructions on how to get setup with
`module development <scripting.html>`_.

Create a new sn0int module like this:

.. code-block:: bash

    sn0int new ~/repos/sn0int-modules/notify-custom.lua

Edit the ``-- Source:`` so it takes notifications as input:

.. code-block:: lua

    -- Description: TODO your description here
    -- Version: 0.1.0
    -- License: GPL-3.0
    -- Source: notifications

    function run(arg)
        -- TODO your code here
        -- https://sn0int.readthedocs.io/en/stable/reference.html

        debug(arg)
        info(arg['subject'])
        info(arg['body'])
    end

Execute your script:

.. code-block:: bash

    sn0int notify exec notify-custom 'hello world'

You most likely need to pass options to avoid hard-coding keys into your
script. Options can be fetched like this:

.. code-block:: lua

    -- Description: TODO your description here
    -- Version: 0.1.0
    -- License: GPL-3.0
    -- Source: notifications

    function run(arg)
        -- TODO your code here
        -- https://sn0int.readthedocs.io/en/stable/reference.html

        local foo = getopt('foo')
        if not foo then return 'Missing -o foo= option' end

        info('foo: ' .. foo)
        info('subject: ' .. arg['subject'])
    end

And passed like this:

.. code-block:: bash

    sn0int notify exec notify-custom -o "foo=hello world" 'ohai'

Setting up notification rules
-----------------------------

We now know how to trigger notifications manually, but we would rather trigger
notifications if a module runs into something interesting.

You can setup subscriptions on specific topics and then have a notification
script execute automatically.

Lookup the location of your sn0int config file:

.. code-block:: bash

    sn0int paths

And open it in an editor of your choice:

.. code-block:: bash

    vim /home/user/.config/sn0int.toml

A basic configuration could look like this:

.. code-block:: toml

    # You can have multiple notification sections, this one is named
    # `demo-telegram-integration`
    # The label can be set to whatever you want, but you may need to add
    # double-quotes to use some characters.
    [notifications.demo-telegram-integration]
    # If this option is present, the notification must originate from one of
    # the following workspaces.
    workspaces = ["default", "some-workspace"]
    # If this option is present, the notification must match one of the
    # filters. You can use `*` as a wildcard to match everything except `:`.
    topics = ["activity:harness/activity-ping:*"]
    # Mandatory: the module to execute.
    script = "kpcyrd/notify-telegram"
    # The options to pass to the module, if any.
    # Can be accessed with `getopt`
    options = [
        "bot_token=1337:foobar",
        "chat_id=1337",
    ]

All options except ``script`` are optional, but setting filters is highly
recommended.

Testing notifications
---------------------

To test if your configuration works correctly you can create an event manually:

.. code-block:: bash

    sn0int -w some-workspace notify send activity:harness/activity-ping:dummy "hello world"

If it matches any of your rules you should receive a push notifications.

.. note::
    If you want to test just the routing without actually sending something, add ``--dry-run``.

Running sn0int automatically
----------------------------

Support for this is going to improve in the future, but you can already set
this up if you're ok with a slightly buggy experience.

Monitors
~~~~~~~~

Some modules are long-running and either wait for an event from a server or
have custom polling built in that's usually configurable with an ``-o
interval=`` option. If your module has a non-trivial setup phase, an author may
take this approach.

.. code-block::

    # /etc/systemd/system/sn0int-your-new-service.service

    [Unit]
    Description=sn0int: run example/changeme

    [Service]
    User=your-user
    ExecStart=/usr/bin/sn0int run -w your-workspace example/changeme

    Restart=always
    RestartSec=0

    [Install]
    WantedBy=multi-user.target

Enable the service to run on boot:

.. code-block:: bash

    systemctl enable --now sn0int-your-new-service.service

Timers
~~~~~~

If the module is only one-shot you can set it up to run with a timer:

.. code-block::

    # /etc/systemd/system/sn0int-your-other-service.service

    [Unit]
    Description=sn0int: run example/changeme

    [Service]
    User=your-user
    ExecStart=/usr/bin/sn0int run -w your-workspace example/changeme

Setup the timer like this:

.. code-block::

    # /etc/systemd/system/sn0int-your-other-service.timer

    [Unit]
    Description=sn0int: run example/changeme

    [Timer]
    OnBootSec=1min
    OnUnitActiveSec=1h

    [Install]
    WantedBy=timers.target

.. code-block:: bash

    systemctl enable --now sn0int-your-other-service.timer
