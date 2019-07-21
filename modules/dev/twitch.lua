-- Description: Collect information from twitch streams
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:twitch.tv

function twitch_get(url)
    local headers = {}
    -- twitch web client id
    headers['Client-ID'] = 'jzkbprff40iqj646a697cyrvl0zt2m6'
    local req = http_request(session, 'GET', url, {
        headers=headers
    })
    return http_send(req)
end

function scrape_panels(uid)
    local headers = {}
    headers['Client-ID'] = 'jzkbprff40iqj646a697cyrvl0zt2m6'
    local req = http_request(session, 'POST', 'https://gql.twitch.tv/gql', {
        headers=headers,
        body='[{"operationName":"ChannelPanels","variables":{"id":"' .. uid .. '"},"extensions":{"persistedQuery":{"version":1,"sha256Hash":"236b0ec07489e5172ee1327d114172f27aceca206a1a8053106d60926a7f622e"}}}]'
    })
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end -- TODO: set_err
    local data = json_decode(r['text'])
    if last_err() then return end

    if not data[1] then return end
    local panels = data[1]['data']['user']['panels']

    for i=1, #panels do
        detect_account(panels[i]['linkURL'])
        harvest_description(panels[i]['description'])
    end
end

function harvest_description(text)
    if not text then return end

    local m = regex_find_all('(http|https)://([\\w_-]+(?:(?:\\.[\\w_-]+)+))([\\w.,@?^=%&:/~+#-]*[\\w@?^=%&/~+#-])?', text)
    for i=1, #m do
        detect_account(m[i][1])
    end

    local m = regex_find_all('[a-zA-Z0-9_\\+\\-\\.]+@[a-zA-Z0-9_\\-\\.]+', text)
    for i=1, #m do
        db_add('email', {
            value=m[i][1],
        })
    end
end

service_map = {}
service_map['www.instagram.com']    = {'^/([^/]+)', 'instagram.com'}
service_map['twitter.com']          = {'^/([^/]+)', 'twitter.com'}
service_map['www.patreon.com']      = {'^/([^/]+)', 'patreon.com'}
service_map['www.tipeeestream.com'] = {'^/([^/]+)', 'tipeeestream.com'}

function detect_account(link)
    if not link then return end
    debug(link)

    local parts = url_parse(link)
    if last_err() then return clear_err() end
    local host = parts['host']

    local service = service_map[host]
    if service then
        local m = regex_find(service[1], parts['path'])
        if m then
            db_add('account', {
                service=service[2],
                username=m[2],
            })
        end
    end
end

function run(arg)
    session = http_mksession()

    update = {}

    local r = twitch_get('https://api.twitch.tv/kraken/channels/' .. arg['username'])
    if last_err() then return end
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end
    local data = json_decode(r['text'])
    if last_err() then return end

    -- info(data)
    uid = data['_id']
    update['displayname'] = data['display_name']
    update['url'] = data['url']
    debug(data['logo']) -- TODO: support linking images as profile pic
    -- debug(data['created_at'])

    local r = twitch_get('https://api.twitch.tv/kraken/streams/' .. arg['username'])
    if last_err() then return end
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end
    local data = json_decode(r['text'])
    if last_err() then return end

    -- info(data)

    if data['stream'] and data['stream']['stream_type'] == 'live' then
        -- data['stream']['created_at']
        debug('online!')
        update['last_seen'] = datetime()
    end

    db_update('account', arg, update)
    if last_err() then return end

    scrape_panels(uid)
end
