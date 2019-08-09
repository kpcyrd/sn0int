-- Description: Discover mail server from MX records for emails
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: emails

function strip_root_dot(name)
    local m = regex_find("(.+)\\.$", name)
    if last_err() then return end

    if m == nil then
        return name
    else
        return m[2]
    end
end

function run(arg)
    -- extract domain
    local domain = arg['value']:match('@(.*)')
    if doman ~= nil then
        -- malformed domain
        return
    end

    -- mx lookup
    local records = dns(domain, {
        record='MX',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']
    -- debug(records)

    for i=1, #records do
        local r = records[i][2]['MX']
        if r then
            local mx = strip_root_dot(r[2])
            domain = psl_domain_from_dns_name(mx)

            domain_id = db_add('domain', {
                value=domain,
            })
            if last_err() then return end

            if domain_id then
                db_add('subdomain', {
                    domain_id=domain_id,
                    value=mx,
                })
                if last_err() then return end
            end
        end
    end
end
