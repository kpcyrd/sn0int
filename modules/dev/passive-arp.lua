-- Description: Passive arp-scanner with sniffglue
-- Version: 0.1.0
-- License: GPL-3.0

-- sudo sniffglue -jv enp0s25

function each_frame(frame)
    if not frame['Ether'] then return end

    local arp = frame['Ether'][2]['Arp']
    if not arp then return end

    if arp['Request'] then
        arp = arp['Request']
    elseif arp['Reply'] then
        arp = arp['Reply']
    else
        -- unknown, abort
        return
    end

    debug(arp)

    -- TODO: this might change to a string in the future
    local mac = mac(arp['src_mac'])
    local ipaddr = arp['src_addr']
    debug({src_mac=mac, src_addr=ipaddr})

    local now = datetime()

    local device_id = db_add('device', {
        value=mac,
        last_seen=now,
    })
    if last_err() then return end

    db_add_ttl('network-device', {
        network_id=network_id,
        device_id=device_id,
        ipaddr=ipaddr,
        last_seen=now,
    }, 120)
    if last_err() then return end
end

function mac(m)
    return
        hex({m[1]}) .. ':' ..
        hex({m[2]}) .. ':' ..
        hex({m[3]}) .. ':' ..
        hex({m[4]}) .. ':' ..
        hex({m[5]}) .. ':' ..
        hex({m[6]})
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
        local x = stdin_readline()
        if x == nil then
            break
        end

        local frame = json_decode(x)
        if last_err() then return end

        each_frame(frame)
        if last_err() then return end
    end
end
