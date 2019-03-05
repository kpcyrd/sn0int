-- Description: Find keybase proofs for online accounts
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts

function run(arg)
    service = arg['service']
    if service == 'twitter.com' then
        service = 'twitter'
    elseif service == 'github.com' then
        service = 'github'
    elseif service == 'reddit.com' then
        service = 'reddit'
    elseif service == 'news.ycombinator.com' then
        service = 'hackernews'
    elseif service == 'facebook.com' then
        service = 'facebook'
    else
        return
    end

    query = {}
    query[service] = arg['username']

    session = http_mksession()
    req = http_request(session, 'GET', 'https://keybase.io/_/api/1.0/user/lookup.json', {
        query=query,
    })
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    x = json_decode(resp['text'])
    if last_err() then return end
    debug(x)

    if x['them'][1] == nil then return end
    them = x['them'][1]

    db_add('account', {
        service='keybase.io',
        username=them['basics']['username'],
    })
end
