-- Description: Query for CNAMES to find subdomains
-- Version: 0.3.0
-- Source: subdomains
-- License: GPL-3.0

function iter(r)
    if r == nil then
        return
    end

    m = regex_find("(.+)\\.$", r)
    if last_err() then return end

    if m == nil then
        return
    end
    r = m[2]

    domain = psl_domain_from_dns_name(r)
    if last_err() then return end

    domain_id = db_add('domain', {
        value=domain,
    })
    if last_err() then return end

    if domain_id ~= nil then
        db_add('subdomain', {
            domain_id=domain_id,
            value=r,
        })
        if last_err() then return end
    end
end

function run(arg)
    records = dns(arg['value'], 'A')
    if last_err() then return end

    if records['error'] ~= nil then return end
    records = records['answers']

    for i=1, #records do
        r = records[i][2]
        iter(r['CNAME'])
        if last_err() then return end
    end
end
