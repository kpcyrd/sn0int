-- Description: Query ThreatMiner passive dns for subdomains of an ip address
-- Version: 0.3.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    session = http_mksession()

    -- TODO: add option to filter old entries based on last_seen

    req = http_request(session, 'GET', 'https://api.threatminer.org/v2/host.php', {
        query={
            rt='2',
            q=arg['value']
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    o = json_decode(resp['text'])
    if last_err() then return end
    o = o['results']

    for i=1, #o do
        x = o[i]

        domain = psl_domain_from_dns_name(x['domain'])
        -- TODO: if this fails, skip this entry instead
        if last_err() then return end

        domain_id = db_add('domain', {
            value=domain,
        })

        if domain_id ~= nil then
            subdomain_id = db_add('subdomain', {
                domain_id=domain_id,
                value=x['domain'],
            })

            db_add('subdomain-ipaddr', {
                subdomain_id=subdomain_id,
                ip_addr_id=arg['id'],
            })
        end
    end
end
