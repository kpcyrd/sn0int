-- Description: Collect information from chefkoch.de profiles
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:chefkoch.de

function trim(text)
    local m = regex_find('^\\s*(.*?)\\s*$', text)
    return m[2]
end

function parse_user_details(html)
    local details = {}

    local rows = html_select_list(html, '#user-details tr')
    if last_err() then return end

    for i=1, #rows do
        -- this is rather annoying, we need to prepend this for the html parser
        local x = html_select_list('<table>' .. rows[i]['html'], 'td')
        if last_err() then return end

        if x[2] then
            local key = trim(x[1]['text'])
            local value = trim(x[2]['text'])
            debug({key, value})
            details[key] = value
        end
    end

    return details
end

function run(arg)
    -- we need a special hash to access chefkoch profiles, it seems this hash is basically a userid
    -- we expect the following format for usernames:
    -- <hex string>/<username>

    if arg['username']:find('/') == nil then
        return 'unexpected username format'
    end

    local session = http_mksession()
    local url = 'https://www.chefkoch.de/user/profil/' .. arg['username'] .. '.html'
    local req = http_request(session, 'GET', url, {})
    local r = http_fetch(req)
    if last_err() then return end

    local update = {}
    local details = parse_user_details(r['text'])

    local member_since = details['Mitglied seit:']
    if member_since then
        -- info(member_since)
        local m = regex_find('([\\d\\.]+)(?: \\(zuletzt online am ([\\d\\.]+)\\))?', member_since)

        local registered = m[2]
        -- TODO: we can't process registered date yet

        local last_online = m[3]
        if last_online then
            last_online = strptime('%d.%m.%Y %H:%M', last_online .. '00:00')
            update['last_seen'] = strftime('%Y-%m-%dT%H:%M:%S', last_online)
        end
    end

    local x = html_select(r['text'], '.username')
    update['displayname'] = trim(x['text'])
    update['url'] = url

    db_update('account', arg, update)
end
