-- Description: Collect information from chaturbate streams
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:chaturbate.com

function resolve_external_link(href)
    local link = url_join('https://chaturbate.com/', href)
    local parts = url_parse(link)
    return parts['params']['url']
end

service_map = {}
service_map['www.instagram.com']    = {'^/([^/]+)', 'instagram.com'}
service_map['twitter.com']          = {'^/([^/]+)', 'twitter.com'}
service_map['www.patreon.com']      = {'^/([^/]+)', 'patreon.com'}
service_map['www.twitch.tv']        = {'^/([^/]+)', 'twitch.tv'}

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
    local session = http_mksession()

    local url = 'https://chaturbate.com/' .. arg['username'] .. '/'
    local req = http_request(session, 'GET', url, {})
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then
        return 'http error: ' .. r['status']
    end

    local last_seen = nil
    if #html_select_list(r['text'], '.offline_tipping') > 0 then
        debug('is offline')
    else
        debug('is online')
        last_seen = datetime()
    end

    db_update('account', arg, {
        last_seen=last_seen,
        url='https://chaturbate.com/' .. arg['username'] .. '/',
    })

    local links = html_select_list(r['text'], 'a[href^="/external_link/"]')
    for i=1, #links do
        local link = resolve_external_link(links[i]['attrs']['href'])
        debug(link)
        detect_account(link)
        if last_err() then return end
    end
end
