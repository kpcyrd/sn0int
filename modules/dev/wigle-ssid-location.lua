-- Description: Use triangulated network location from wardriving data
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: networks
-- Keyring-Access: wigle

function run(arg)
    local key = keyring('wigle')[1]
    if not key then
        return 'wigle api key is required'
    end

    local session = http_mksession()
    local url = 'https://api.wigle.net/api/v2/network/search'
    local req = http_request(session, 'GET', url, {
        basic_auth={key['access_key'], key['secret_key']},
        query={
            ssid=arg['value'],
        }
    })
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then
        return 'unexpected status code: ' .. r['status']
    end

    local data = json_decode(r['text'])
    if last_err() then return end

    if not data['success'] then
        return "api didn't signal success"
    end

    -- skip if ssid isn't unique
    if data['totalResults'] ~= 1 then
        return
    end

    -- ssid is globally unique
    local network = data['results'][1]
    debug(network)

    -- the location is triangulated, only use this if we don't overwrite anything
    if not arg['latitude'] and not arg['longitude'] then
        db_update('network', arg, {
            latitude=network['trilat'],
            longitude=network['trilong'],
        })
    end
end
