-- Description: Query ThreatMiner passive dns for subdomains of a domain
-- Version: 0.1.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    session = http_mksession()

    req = http_request(session, 'GET', 'https://api.threatminer.org/v2/domain.php', {
        query={
            rt='5',
            q=arg['value']
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    o = json_decode(resp['text'])
    if last_err() then return end
    o = o['results']

    i = 1
    while o[i] do
        x = o[i]

        db_add('subdomain', {
            domain_id=arg['id'],
            value=x,
        })

        i = i+1
    end
end
