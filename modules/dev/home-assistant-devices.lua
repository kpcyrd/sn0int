-- Description: Query device location from home assistant
-- Version: 0.1.0
-- License: GPL-3.0
-- Keyring-Access: home-assistant

function run()
    -- parsing options
    instance = getopt('instance')
    if not instance then
        return 'instance option is missing'
    end

    host = url_parse(instance)
    if last_err() then return end
    host = host['host']

    entity = getopt('entity')
    if not entity then
        return 'entity option is missing'
    end

    -- fetching credentials
    creds = keyring('home-assistant:' .. host)
    if creds[1] == nil then
        profile = url_join(instance, 'profile')
        return 'missing home-assistant:' .. host .. ' Long-Lived Access Token, open ' .. profile
    end
    token = creds[1]['secret_key']

    headers = {}
    headers['Authorization'] = 'Bearer ' .. token
    headers['Content-Type'] = 'application/json'

    -- requesting status
    session = http_mksession()
    url = url_join(instance, 'api/states/' .. entity)
    req = http_request(session, 'GET', url, {
        headers=headers
    })
    r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then
        return 'http error: ' .. r['status']
    end

    m = json_decode(r['text'])
    if last_err() then return end
    debug(m)

    info({
        gps_accuracy=m['attributes']['gps_accuracy'],
        longitude=m['attributes']['longitude'],
        latitude=m['attributes']['latitude'],
        last_updated=m['last_updated'],
    })
end
