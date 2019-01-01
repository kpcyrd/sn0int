-- Description: Parse iw station dump
-- Version: 0.1.0
-- License: GPL-3.0

-- iw dev wlan0 station dump

function add(client)
    if
        client['authenticated'] == 'yes' and
        client['mac']
    then
        info(client)
    end

    client = nil
end

function each_line(x)
    debug(x)
    m = regex_find('^Station (\\S+)', x)
    if m then
        if client then
            add(client)
        end
        client = {}
        client['mac'] = m[2]
        debug('mac=' .. m[2])
    end

    m = regex_find('^\\s+([^:]+):\\s*(.+)\n$', x)
    if m and client then
        client[m[2]] = m[3]
        debug(m[2] .. '=' .. m[3])
    end
end

function run()
    client = nil
    while true do
        x = stdin_readline()
        if x == nil then
            break
        end

        each_line(x)
    end

    if client then
        add(client)
    end
end
