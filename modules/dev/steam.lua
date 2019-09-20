-- Description: Collection information from steam profiles
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:steamcommunity.com

function get_last_seen(html)
    local x = html_select_list(html, '.profile_header .offline')
    if last_err() then return end
    if #x > 0 then
        -- TODO: try to read last_online: $$('.profile_in_game_name')
        debug('offline')
    else
        debug('online')
        return datetime()
    end
end

function get_displayname(html)
    local x = html_select_list(html, '.header_real_name bdi')
    if #x > 0 and x[1]['text'] ~= "" then
        return x['text']
    else
        x = html_select(html, '.actual_persona_name')
        if last_err() then return end
        return x['text']
    end
end

function download_avatar(html)
    local img = html_select(html, '.playerAvatarAutoSizeInner img')
    if last_err() then return end

    local url = img['attrs']['src']
    local req = http_request(session, 'GET', url, {
        into_blob=true,
    })
    local r = http_fetch(req)
    if last_err() then return end

    db_add('image', {
        value=r['blob'],
    })
    return r['blob']
end

--[[
function get_profiles(html)
    -- try to discover profiles
    local links = html_select_list(html, '.profile_summary a')
    if last_err() then return end
    for i=1, #links do
        -- info(links[i]['href'])
    end
end
]]--

function run(arg)
    local url = 'https://steamcommunity.com/id/' .. arg['username']

    session = http_mksession()
    local req = http_request(session, 'GET', url, {})
    local r = http_fetch(req)
    if last_err() then return end

    -- fall back to steam userids in case of an error
    local x = html_select_list(r['text'], '.error_ctn')
    if #x > 0 then
        url = 'https://steamcommunity.com/profiles/' .. arg['username']
        req = http_request(session, 'GET', url, {})
        r = http_fetch(req)
        if last_err() then return end
    end

    update = {}
    update['url'] = url
    update['last_seen'] = get_last_seen(r['text'])
    if last_err() then return end
    update['displayname'] = get_displayname(r['text'])
    if last_err() then return end
    update['profile_pic'] = download_avatar(r['text'])
    if last_err() then return end

    db_update('account', arg, update)
end
