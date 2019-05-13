-- Description: Add a domains NS records to scope
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains

function strip_root_dot(name)
    local m = regex_find("(.+)\\.$", name)
    if last_err() then return end

    if m == nil then
        return name
    else
        return m[2]
    end
end

function each(r)
    local name = strip_root_dot(r)
    local domain = psl_domain_from_dns_name(name)
    if last_err() then return end

    -- add domain
    local domain_id = db_add('domain', {
        value=domain,
    })
    if last_err() then return end
    if domain_id == nil then return end

    -- add subdomain
    local subdomain_id = db_add('subdomain', {
        domain_id=domain_id,
        value=name,
    })
    if last_err() then return end
end

function run(arg)
    local records = dns(arg['value'], {
        record='NS',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']

    for i=1, #records do
        local r = records[i][2]
        debug(r)
        each(r['NS'])
        if last_err() then return end
    end
end
