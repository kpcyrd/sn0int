-- Description: Discover subdomains from wayback machine
-- Version: 0.1.0
-- Source: domains

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
    if o[0] == nil then
        return
    end

    -- ensure the api response is still what we expect
    if o[0][2] == nil then
        return 'api returned unexpected json format'
    end

    seen = {}

    i = 1
    while o[i] do
        url = o[i][2]

        parts = url_parse(url)

        if last_err() then
            clear_err()
            error("Failed to parse url: " .. json_encode(url))
        else
            subdomain = parts['host']

            if seen[subdomain] == nil then
                db_add('subdomain', {
                    domain_id=arg['id'],
                    value=parts['host'],
                })
                if last_err() then return end
                seen[subdomain] = 1
            end
        end

        i = i+1
    end
end
