-- Description: Collect data from instagram profiles
-- Version: 0.1.0
-- Source: accounts:instagram.com
-- License: GPL-3.0

function run(arg)
    local session = http_mksession()
    local url = 'https://www.instagram.com/' .. arg['username'] .. '/'
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'invalid status code' end
    local html = resp['text']

    local ld = html_select(html, 'script[type="application/ld+json"]')
    if last_err() then return end
    debug(ld)

    local ld = json_decode(ld['text'])
    if last_err() then return end
    debug(ld)

    if ld['email'] then
        db_add('email', {
            value=ld['email'],
        })
    end

    -- homepage=ld['url']

    db_update('account', arg, {
        displayname=ld['name'],
        email=ld['email'],
        url=url,
    })
end
