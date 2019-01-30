-- Description: Parse isc-dhcpd dhcpd.leases(5)
-- Version: 0.2.0
-- License: GPL-3.0

-- cat /var/lib/dhcpd/dhcpd.leases

function add(lease)
    if not lease['active'] then return end

    now = datetime()

    device_id = db_add('device', {
        value=lease['mac'],
        hostname=lease['hostname'],
        last_seen=now,
    })
    if last_err() then return end

    db_add_ttl('network-device', {
        network_id=network_id,
        device_id=device_id,
        ipaddr=lease['ipaddr'],
        last_seen=now,
    }, 180)
    if last_err() then return end
end

function each_line(x)
    debug(x)
    m = regex_find('^lease (\\S+) \\{\n$', x)
    if m then
        lease = {}
        debug('ipaddr=' .. m[2])
        lease['ipaddr'] = m[2]
    end
    m = regex_find('^\\s*hardware ethernet (\\S+);\n$', x)
    if m then
        debug('mac=' .. m[2])
        lease['mac'] = m[2]
    end
    m = regex_find('^\\s*client-hostname \"(.+)\";\n$', x)
    if m then
        debug('hostname=' .. m[2])
        lease['hostname'] = m[2]
    end
    m = regex_find('^\\s*binding state active;\n$', x)
    if m then
        debug('active=true')
        lease['active'] = true
    end
    m = regex_find('^\\}\n$', x)
    if m then
        add(lease)
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

    while true do
        x = stdin_readline()
        if x == nil then
            break
        end

        if not regex_find('^\\s*(#.*|\\s*)\n$', x) then
            each_line(x)
        end
    end
end
