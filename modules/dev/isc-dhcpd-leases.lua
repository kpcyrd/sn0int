-- Description: Parse isc-dhcpd dhcpd.leases(5)
-- Version: 0.1.0
-- License: GPL-3.0

-- cat /var/lib/dhcpd/dhcpd.leases

function add(lease)
    info({lease=lease})

    device = {
        value=lease['mac'],
        hostname=lease['hostname'],
    }
    info({device=device})

    -- TODO: network_id
    -- TODO: device_id
    network_device = {
        ipaddr=lease['ipaddr']
    }
    info({network_device=network_device})
end

function each_line(x)
    debug(x)
    m = regex_find('^lease (\\S+) \\{\n$', x)
    if m then
        info('# reset')
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
    end
    m = regex_find('^\\}\n$', x)
    if m then
        add(lease)
    end
end

function run()
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
