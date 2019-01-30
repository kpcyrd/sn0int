-- Description: Scan subdomains for websites
-- Version: 0.3.0
-- Source: subdomains
-- License: GPL-3.0

function request(subdomain_id, url)
    req = http_request(session, 'GET', url, {
        timeout=5000
    })
    reply = http_send(req)

    if last_err() then
        clear_err()
        return
    end

    obj = {
        subdomain_id=subdomain_id,
        value=url,
        status=reply['status'],
        body=reply['text'],
    }

    redirect = reply['headers']['location']
    if redirect then
        obj['redirect'] = url_join(url, redirect)
    end

    db_add('url', obj)

    -- debug(reply['status'])
    -- debug(reply['headers']['location'])
    -- debug(reply['text'])
end

function run(arg)
    domain = arg['value']

    session = http_mksession()

    request(arg['id'], 'http://' .. domain .. '/')
    if last_err() then return end
    request(arg['id'], 'https://' .. domain .. '/')
    if last_err() then return end
end
