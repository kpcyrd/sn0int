Writing your first module
=========================

Scripting is the core feature in sn0int. It's not strictly required, but if you
want to write your own modules, this section is for you.

Creating a repository
---------------------

It's highly recommended to use a VCS for development, so let's start by setting
that up. We're going to assume you store your repos in ``~/repos`` but you're
free to change that to something else::

    $ git init ~/repos/sn0int-modules
    $ cd ~/repos/sn0int-modules

.. note::
   If you're using github you can also create a repo from the `module repo
   template`_.

.. _module repo template: https://github.com/sn0int/sn0int-modules

We need to add this folder to the sn0int config file so it's correctly detected
when starting sn0int. Open the `config file <config.html>`_ in your prefered
editor. Note that the file does not exist by default and the path is different
depending on your operating system. On linux you would open the config file
with::

    $ vim ~/.config/sn0int.toml

Add the following::

    [namespaces]
    your_github_name = "~/repos/sn0int-modules"

Every module we're adding to ``~/repos/sn0int-modules`` is now going to be
picked up by sn0int.

Make sure you're still in the right folder and add your first module::

    sn0int new first.lua

This is going to generate some boilerplate for you that every module needs to
load successfully. Afterwards we can edit it like this:

.. code-block:: lua

    -- Description: ohai wurld
    -- Version: 0.1.0
    -- Source: domains
    -- License: GPL-3.0

    function run(arg)
        -- TODO: do something here
    end

``Description`` (mandatory)
  This should be a short text that describes what your module is doing.

``Version`` (mandatory)
  Every module requires a semver_ version. You can just set it to ``0.1.0``
  during development, but you need to increase it every time you publish your
  module. If you don't care about that one, just keep increasing ``0.X.0``.

.. _semver: https://semver.org/

``Source`` (mandatory)
  This is going to specify what kind of entities we're interested in. If we
  specify ``domains`` our module is going to be called with all domains that
  are targeted.

  - ``domains``
  - ``subdomains``
  - ``ipaddrs``
  - ``urls``
  - ``emails``

``License`` (mandatory)
  This is somewhat special. We require that every module is licensed under an
  open source license. Pick one of the following licenses.

  - ``MIT`` -  https://opensource.org/licenses/MIT
  - ``GPL-3.0`` - https://opensource.org/licenses/gpl-license
  - ``LGPL-3.0`` - https://opensource.org/licenses/lgpl-license
  - ``BSD-2-Clause`` - https://opensource.org/licenses/BSD-2-Clause
  - ``BSD-3-Clause`` - https://opensource.org/licenses/BSD-3-Clause
  - ``WTFPL`` - https://spdx.org/licenses/WTFPL.html

``function run(arg)`` (mandatory)
  This is where the actual magic of our module happens. Our function is going
  to be called in a loop for each entity that is targeted by the user.

Let's continue. For the sake of an hello world we're going to take some
``domains``, check if a ``www`` subdomain exists and if it does, add it to the
database.

.. code-block:: lua

    -- Description: Scan for www. subdomains
    -- Version: 0.1.0
    -- Source: domains
    -- License: GPL-3.0

    function run(arg)
        subdomain = 'www.' .. arg['value']
        info(subdomain)
    end

This is already enough to execute it. Make sure you've added a domain to scope
with ``add domain example.com``, save your file and run it like this::

    sn0int run -f ./first.lua

We should see some output by our info function.

.. note::
   ``info`` is useful for development but you usually want your module to run
   quietly, so before publishing either remove it or replace it with ``debug``.

Next, we want to actually resolve that name, we're going to use the ``dns``
function for that. This function takes a name and a query type and returns a
result. Note that this function might fail, in which case we want to abort our
function. We do that by checking if the return value of ``last_err()`` is
truth-y.

.. code-block:: lua

    -- Description: Scan for www. subdomains
    -- Version: 0.1.0
    -- Source: domains
    -- License: GPL-3.0

    function run(arg)
        subdomain = 'www.' .. arg['value']

        records = dns(subdomain, {
            record='A'
        })
        if last_err() then return end

        info(records)
    end

If you run your module again you're going to see some output, either
``{"answers":[somedata],"error":null}`` or
``{"answers":[],"error":"NXDomain"}``. If the dns reply doesn't indicate an
error this means the subdomain exists and we can add it to our database with
``resolvable`` being set to ``true``.

.. code-block:: lua

    -- Description: Scan for www. subdomains
    -- Version: 0.1.0
    -- Source: domains
    -- License: GPL-3.0

    function run(arg)
        subdomain = 'www.' .. arg['value']

        records = dns(subdomain, {
            record='A'
        })
        if last_err() then return end

        if records['error'] == nil then
            db_add('subdomain', {
                domain_id=arg['id'],
                value=subdomain,
                resolvable=true,
            })
        end
    end

.. hint::
   See the database section to understand how the database works in detail.

If we execute our finished module one more time it's going to log that it
discovered a subdomain, if it doesn't, try adding more domains to scope. Note
that this only happens the first time. Modules that don't discover anything or
don't discover anything new exit silently.

There's still some room for improvement, for example, since we already resolved
that record, we could also add the ip address to the scope and link it to the
subdomain we added.

.. hint::
   For debugging purposes you can increase the verbosity with ``sn0int run -v``
   so database operations are logged even if nothing was changed, or with
   ``sn0int run -vv`` to enable ``debug()`` output.

Publish your module
-------------------

The public registry uses github usernames to namespace the registry. This means
you need to authenticate to the registry using your github username. This can
be done using::

    sn0int login

sn0int is going to open a new tab in your browser, if you are already signed
into your github account you only need to confirm an authorization request. The
application doesn't need any of your data, so it's only asking you to confirm
your identity.

Afterwards publish your module with::

    sn0int publish ./first.lua

Please also make sure you publish your repository to github so other people can
submit pull requests. The recommended repository location is::

    https://github.com/<your-username>/sn0int-modules

Publish your repo
-----------------

It is highly recommended to publish your repository on github so people can
file issues and pull requests for your module. If you've been following along
with the github template you can simply commit your changes and push them.

Your repository would look like one of these:

- https://github.com/kpcyrd/sn0int-modules
- https://github.com/ysf/sn0int-modules
- https://github.com/cybiere/sn0int-modules

Reading data from stdin
-----------------------

Sometimes you need to read data that can't be easily accessed from within the
sandbox, like output of other programms or file content. In that case you can
write a module that reads from stdin:

.. code-block:: lua

    -- Description: Read from stdin
    -- Version: 0.1.0
    -- License: GPL-3.0

    function run()
        while true do
            x = stdin_readline()
            if x == nil then
                break
            end
            info(x)
        end
    end

Write it to a file and run it like this::

    % echo hello | sn0int run --stdin -vvf stdin.lua
    [*] anonymous/stdin                                   : "hello\n"
    [+] Finished anonymous/stdin
    %

This is going to read one line at a time and allows you to process it with
regular expressions and add data to the database.

.. note::
   If you get an error like ``Failed to read stdin: "stdin is unavailable"``
   make sure the ``--stdin`` flag is set.
