Sandbox
=======

Scripts are generally considered to be untrusted and executed exclusively in a
child process. It's important to note that there's a basic sandbox that's
active on every operating system, and there's a second line of defense on
supported operating systems.

The first line of defense is the restrictive stdlib. It's assumed that an
attacker gains full control over the lua code and is able to call any function
with arbitrary arguments. The stdlib only provides functions that are
considered safe, so for example it's not possible to start a process or open a
file.

The second line of defense is supposed to make sure the system isn't
compromised even if the first layer is fully broken and an attacker gains full
control over the child process.

Right now this is only supported on linux and openbsd.

Linux
-----

On linux we use seccomp to filter all syscalls that we don't need. We also use
chroot to disable filesystem access. It's recommended to install the sn0int
binary with ``cap_sys_chroot`` to make sure unprivileged users can use chroot.
The chroot location is hard coded and all capabilities are removed after the
chroot is done or if no chroot is going to happen.

OpenBSD
-------

On openbsd we're using ``pledge`` to restrict syscalls and ``unveil`` to
restrict filesystem access.

IPC Protocol
------------

The parent process and the child process communicate using an IPC protocol that
is line-based json.

For a simple hello world the parent process is only going to send a single line
to the child process. This line contains:

- The function argument
- The dns config
- Keys that the module has been given access to
- The module metadata and code
- Options, if any
- A socks5 proxy, if any
- The log level

.. code-block:: json

    {"arg":null,"dns_config":{"ns":["1.1.1.1:53","1.0.0.1:53"],"tcp":false,"timeout":{"nanos":0,"secs":3}},"keyring":[],"module":{"author":"anonymous","description":"basic selftest","keyring_access":[],"name":"selftest","script":{"code":"-- Description: basic selftest\n-- Version: 0.1.0\n-- License: GPL-3.0\n\nfunction run()\n    -- nothing to do here\nend\n"},"source":null,"version":"0.1.0"},"options":{},"proxy":null,"verbose":2}

Saving this line in a file called ``start.json`` and sending it to a sandbox
process should result in the following output::

    $ sn0int sandbox foobar < start.json
    {"Exit":"Ok"}
    $

This line tells us that the script terminated successfully.

There are some functions that cause a notification to the parent process. We
are going to add a call to the ``info()`` function to our module:

.. code-block:: json

    {"arg":null,"dns_config":{"ns":["1.1.1.1:53","1.0.0.1:53"],"tcp":false,"timeout":{"nanos":0,"secs":3}},"keyring":[],"module":{"author":"anonymous","description":"basic selftest","keyring_access":[],"name":"selftest","script":{"code":"-- Description: basic selftest\n-- Version: 0.1.0\n-- License: GPL-3.0\n\nfunction run()\n    info('ohai')\nend\n"},"source":null,"version":"0.1.0"},"options":{},"proxy":null,"verbose":2}

This is going to print an additional event::

    $ sn0int sandbox foobar < start2.json
    {"Log":{"Info":"\"ohai\""}}
    {"Exit":"Ok"}
    $

There are some functions that block the child process until the parent process
sent a reply. These functions are mostly database related functions, since the
child doesn't have direct database access. To demonstrate this, we're going to
write two lines to our file this time, one is the init line and the second one
is the reply for the database event:

.. code-block:: json

    {"arg":null,"dns_config":{"ns":["1.1.1.1:53","1.0.0.1:53"],"tcp":false,"timeout":{"nanos":0,"secs":3}},"keyring":[],"module":{"author":"anonymous","description":"basic selftest","keyring_access":[],"name":"selftest","script":{"code":"-- Description: basic selftest\n-- Version: 0.1.0\n-- License: GPL-3.0\n\nfunction run()\n    x = db_add('domain', {value=\"example.com\"})\n    info(x)\nend\n"},"source":null,"version":"0.1.0"},"options":{},"proxy":null,"verbose":2}
    {"Ok":1337}

Results in the following output::

    $ target/release/sn0int sandbox foobar < start3.json
    {"Database":{"Insert":{"Domain":{"value":"example.com"}}}}
    {"Log":{"Info":"1337.0"}}
    {"Exit":"Ok"}
    $

The first line is a database event and indicates that the child wants to insert
data. After printing this line the child tries to read a line from stdin, this
is why we needed to write two lines to our json file this time. In the second
line the child learns if the insert was successful and which id was assigned to
that entity.

Limitations
-----------

There are some limitations that you should be aware:

- Network access is available and network namespaces aren't isolated. This
  means scripts have access to your local network, the internet and also your
  localhost loopback interface.
- If chroot is unavailable an attacker could connect to unix domain sockets.

Diagnosing a sandbox failure
----------------------------

You might experience a sandbox failure, especially on architectures that are
less popular. This usually looks like this::

    [sn0int][example][kpcyrd/ctlogs] > run
    [-] Failed "example.com": Sandbox child has crashed
    [+] Finished kpcyrd/ctlogs (1 errors)

A module that never finishes could also mean an IO thread inside the worker got
killed by the sandbox.

You can try to diagnose this yourself with strace::

    strace -f sn0int run -vv ctlogs 2>&1 | tee strace.log

Open ``strace.log``, look out for syscalls that didn't return by searching for
``= ?`` and ignore calls to exit and similar. You are looking for something
like this::

    seccomp(SECCOMP_SET_MODE_FILTER, 0, {len=48, filter=0xdd59094e490}) = 0
    write(1, "[+] activated!\n", 15[+] activated!
    )        = 15
    getresuid( <unfinished ...>)            = ?
    +++ killed by SIGSYS (core dumped) +++

This would indicate a call to ``getresuid`` which was not allowed by the
seccomp filter.

If you don't want to diagnose this yourself open a new bug report with as much
information as possible, specifically which distro, which release and which
architecture you're using.
