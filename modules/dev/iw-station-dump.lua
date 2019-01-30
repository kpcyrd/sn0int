-- Description: Parse iw station dump
-- Version: 0.2.0
-- License: GPL-3.0

-- iw dev wlan0 station dump

function add(client)
    if
        client['authenticated'] == 'yes' and
        client['authorized'] == 'yes' and
        client['mac']
    then
        debug(client)

        now = datetime()

        device_id = db_add('device', {
            value=client['mac'],
            last_seen=now,
        })
        if last_err() then return end

        db_add_ttl('network-device', {
            network_id=network_id,
            device_id=device_id,
            last_seen=now,
        }, 180)
        if last_err() then return end
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
    network = getopt('network')
    if not network then
        return 'network option is missing'
    end

    network_id = db_select('network', network)
    if not network_id then
        return 'network not found in database'
    end

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
