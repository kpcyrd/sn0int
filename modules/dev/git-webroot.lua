-- Description: Search for git checkouts in webroot
-- Version: 0.1.0
-- Source: urls
-- License: GPL-3.0

function run(arg)
    url = url_join(arg['value'], '.git/HEAD')

    session = http_mksession()
    req = http_request(session, 'GET', url, {})
    reply = http_send(req)
    if last_err() then return end

    if reply['status'] ~= 200 then
        return
    end

    if not regex_find('^ref: ', reply['text']) then
        return
    end

    db_add('url', {
        subdomain_id=arg['subdomain_id'],
        value=url,
        status=reply['status'],
        body=reply['text'],
    })
end
