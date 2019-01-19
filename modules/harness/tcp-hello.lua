-- Description: Send a hello to a server on port 1337
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    addr = getopt('addr')
    if not addr then
        return 'addr is not set'
    end

    -- create connection
    c = sock_connect(addr, 1337, {})
    if last_err() then return end

    -- send ohai
    sock_sendline(c, 'ohai')
    if last_err() then return end
end
