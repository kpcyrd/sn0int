-- Description: Scan for known /.well-known/ locations
-- Version: 0.2.0
-- Source: urls
-- License: GPL-3.0

function run(arg)
    -- https://www.iana.org/assignments/well-known-uris/well-known-uris.xhtml
    -- https://en.wikipedia.org/wiki/List_of_/.well-known/_services_offered_by_webservers

    -- TODO: check if every location causes a 200/redirect

    locations = {
        {path='security.txt'}, -- expect 200
        {path='dnt-policy.txt'}, -- expect 200
        {path='caldav', redirect=true}, -- expect redirect
        {path='autoconfig/mail/config-v1.1.xml'}, -- expect 200
        {path='assetlinks.json'}, -- expect 200
        {path='apple-app-site-association'},  -- expect 200
        {path='keybase.txt'}, -- expect 200
        {path='apple-developer-merchantid-domain-association'}, -- expect 200
        {path='openpgpkey'}, -- expect 200
        {path='change-password', redirect=true}, -- expect redirect
    }

    session = http_mksession()

    for i=1, #locations do
        path = locations[i]['path']
        expect_redirect = locations[i]['redirect']

        url = url_join(arg['value'], '/.well-known/' .. path)
        debug(url)

        req = http_request(session, 'GET', url, {
            timeout=5000,
        })
        reply = http_send(req)
        debug(reply)

        if last_err() then
            clear_err()
        else
            status = reply['status']
            if (status == 200 and not expect_redirect) or (expect_redirect and status >= 300 and status < 400) then
                obj = {
                    subdomain_id=arg['subdomain_id'],
                    value=url,
                    status=reply['status'],
                    body=reply['text'],
                }

                redirect = reply['headers']['location']
                if redirect then
                    obj['redirect'] = url_join(url, redirect)
                end

                db_add('url', obj)
            end
        end
    end
end
