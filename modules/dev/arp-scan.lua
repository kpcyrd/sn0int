-- Description: Parse arp-scan output
-- Version: 0.3.0
-- License: GPL-3.0

-- sudo arp-scan -qglI wlp3s0

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

        m = regex_find('(.+)\t(.+)', x)
        if m ~= nil then
            ipaddr = m[2]
            mac = m[3]
            now = datetime()

            device_id = db_add('device', {
                value=mac,
                last_seen=now,
            })
            if last_err() then return end

            db_add_ttl('network-device', {
                network_id=network_id,
                device_id=device_id,
                ipaddr=ipaddr,
                last_seen=now,
            }, 300)
            if last_err() then return end
        end
    end
end
