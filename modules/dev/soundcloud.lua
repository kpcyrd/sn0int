-- Description: Collect information from soundcloud pages
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:soundcloud.com

service_map = {}
service_map['www.instagram.com']    = {'^/([^/]+)', 'instagram.com'}
service_map['www.facebook.com']     = {'^/([^/]+)', 'facebook.com'}
service_map['www.mixcloud.com']     = {'^/([^/]+)', 'mixcloud.com'}

function detect_account(link)
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
    local client_id = 'KcozVmRIF7YgS1UmXoWoSwPz3TWQ9TU7'
    local url = 'https://api.soundcloud.com/users/' .. arg['username'] .. '?client_id=' .. client_id

    local session = http_mksession()
    local req = http_request(session, 'GET', url, {})
    local r = http_fetch_json(req)
    if last_err() then return end

    -- info(r)

    --[[
    TODO: use this for last_seen
    local last_modified = strptime('%Y/%m/%d %H:%M:%S +0000', r['last_modified'])
    info(last_modified)
    ]]--

    local update = {}
    update['url'] = r['permalink_url']
    if r['full_name'] and r['full_name'] ~= "" then
        -- full_name is technically not the display name
        update['displayname'] = r['full_name']
    end
    -- TODO: extract avatar

    db_update('account', arg, update)

    url = 'https://api-v2.soundcloud.com/users/soundcloud:users:' .. r['id'] .. '/web-profiles?client_id=' .. client_id
    req = http_request(session, 'GET', url, {})
    r = http_fetch_json(req)
    if last_err() then return end

    for i=1, #r do
        detect_account(r[i]['url'])
        if last_err() then return end
    end
end
