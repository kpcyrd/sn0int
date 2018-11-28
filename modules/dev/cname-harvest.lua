-- Description: Query for CNAMES to find subdomains
-- Version: 0.1.0
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

    -- there is a bug in struct -> lua that causes tables to be zero indexed
    -- this checks if there's something at index 0 but uses index 1 if this is fixed
    i = 0
    if records[i] == nil then i = 1 end

    while records[i] ~= nil do
        r = records[i]['CNAME']
        iter(r)
        if last_err() then return end
        i = i+1
    end
end
