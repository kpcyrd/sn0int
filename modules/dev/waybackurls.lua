-- Description: Discover subdomains from wayback machine
-- Version: 0.4.0
-- Source: domains
-- License: GPL-3.0

function run(arg)

    domain = arg['value']
    url = 'https://web.archive.org/cdx/search/cdx?url=*.' .. domain .. '/*&output=json&collapse=urlkey'

    session = http_mksession()
    req = http_request(session, 'GET', url, {})
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    o = json_decode(resp['text'])
    if last_err() then return end

    -- no known urls
    if o[1] == nil then
        return
    end

    -- ensure the api response is still what we expect
    if o[1][3] == nil then
        return 'api returned unexpected json format'
    end

    seen = {}

    for i=2, #o do
        url = o[i][3]
        debug(url)
        parts = url_parse(url)

        if last_err() then
            clear_err()
            error("Failed to parse url: " .. json_encode(url))
        else
            subdomain = parts['host']
            subdomain, _ = subdomain:gsub('%.$', '')

            if seen[subdomain] == nil then
                db_add('subdomain', {
                    domain_id=arg['id'],
                    value=parts['host'],
                })
                if last_err() then return end
                seen[subdomain] = 1
            end
        end
    end
end
