-- Description: Query hackertarget for subdomains of a domain
-- Version: 0.2.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    session = http_mksession()

    req = http_request(session, 'GET', 'https://api.hackertarget.com/hostsearch/', {
        query={
            q=arg['value']
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    m = regex_find_all("([^,]+),.+\\n?", resp['text'])

    for i=1, #m do
        db_add('subdomain', {
            domain_id=arg['id'],
            value=m[i][2]
        })
    end
end
