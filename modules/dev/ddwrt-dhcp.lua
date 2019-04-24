-- Description: Export dhcp leases from ddwrt webinterface
-- Version: 0.2.0
-- License: GPL-3.0

function run()
    network = getopt('network')
    if not network then
        return 'network option is missing'
    end

    network_id = db_select('network', network)
    if not network_id then
        return 'network not found in database'
    end

    skip_redacted = not getopt('use-redacted')

    router = getopt('router') -- http://192.0.2.1/
    if not router then
        return 'router option is missing (http://192.0.2.1/)'
    end
    username = getopt('user')
    password = getopt('password')

    options = {}
    if username and password then
        options['basic_auth'] = {username, password}
    end

    -- request status page
    session = http_mksession()
    url = url_join(router, '/Info.live.htm')
    req = http_request(session, 'GET', url, options)
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then
        return 'http error: ' .. resp['status']
    end

    txt = resp['text']
    debug(txt)

    -- extract leases from response
    dhcp_section = regex_find('\\{dhcp_leases:: ([^\\}]+)\\}', txt)
    if last_err() then return end
    if not dhcp_section then
        return 'Failed to get dhcp lease section'
    end

    leases = regex_find_all('\'([^\']+)\',\'([^\']+)\',\'([^\']+)\',\'[^\']+\',\'[^\']+\'', dhcp_section[2])
    if last_err() then return end

    now = datetime()

    -- add devices to database
    for i=1, #leases do
        local hostname = leases[i][2]
        local ipaddr = leases[i][3]
        local macaddr = leases[i][4]

        debug({
            hostname=hostname,
            ipaddr=ipaddr,
            macaddr=macaddr,
        })

        if skip_redacted and macaddr:match('^xx:xx:') then
            info('Skipping redacted macaddr')
        else
            local device = {
                value=macaddr,
                last_seen=now,
            }
            if hostname ~= '*' then
                device['hostname'] = hostname
            end

            local device_id = db_add('device', device)

            db_add_ttl('network-device', {
                network_id=network_id,
                device_id=device_id,
                ipaddr=ipaddr,
                last_seen=now,
            }, 120)
        end
    end
end
