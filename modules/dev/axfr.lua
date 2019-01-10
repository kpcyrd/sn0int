-- Description: Try a zone transfer for subdomains
-- Version: 0.2.0
-- Source: domains
-- License: GPL-3.0

function strip_root_dot(name)
    m = regex_find("(.+)\\.$", name)
    if last_err() then return end

    if m == nil then
        return name
    else
        return m[2]
    end
end

function add_pointer(name)
    local domain, domain_id, subdomain_id

    -- select psl+1
    domain = psl_domain_from_dns_name(name)
    if last_err() then return end

    -- add domain
    domain_id = db_add('domain', {
        value=domain,
    })
    if last_err() then return end
    if domain_id == nil then return end

    -- add subdomain
    subdomain_id = db_add('subdomain', {
        domain_id=domain_id,
        value=name,
    })
    if last_err() then return end
end

function iter_axfr(zone, arg)
    local name, r, m, domain

    debug(arg)

    name = arg[1]
    r = arg[2]

    -- select psl+1
    domain = psl_domain_from_dns_name(name)
    if last_err() then return end

    -- add domain
    domain_id = db_add('domain', {
        value=domain,
    })
    if last_err() then return end
    if domain_id == nil then return end

    -- add subdomain
    subdomain_id = db_add('subdomain', {
        domain_id=domain_id,
        value=name,
    })
    if last_err() then return end

    -- this is a A record
    if r['A'] ~= nil then
        -- add the name and ip
        ipaddr_id = db_add('ipaddr', {
            family='4',
            value=r['A'],
        })
        if last_err() then return end

        db_add('subdomain-ipaddr', {
            subdomain_id=subdomain_id,
            ip_addr_id=ipaddr_id,
        })
        if last_err() then return end
    end

    if r['CNAME'] ~= nil then
        -- add the name and the name it's pointing to
        name = strip_root_dot(r['CNAME'])
        add_pointer(name)
    end

    if r['NS'] ~= nil then
        -- add the name and the name it's pointing to
        name = strip_root_dot(r['NS'])
        add_pointer(name)
    end

    if r['MX'] ~= nil then
        -- add the name and the name it's pointing to
        name = strip_root_dot(r['MX'][2])
        add_pointer(name:lower())
    end
end

function iter_a(zone, arg)
    local i, records, r

    if arg == nil then return end

    debug('nameserver: ' .. arg)
    records = dns(zone, {
        record='AXFR',
        nameserver=arg .. ':53',
        tcp=true,
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']

    i = 1
    while records[i] ~= nil do
        iter_axfr(zone, records[i])
        if last_err() then return end
        i = i+1
    end
end

function iter_ns(zone, arg)
    local i, records, r

    if arg == nil then return end

    records = dns(arg, {
        record='A',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']

    i = 1
    while records[i] ~= nil do
        r = records[i][2]
        iter_a(zone, r['A'])
        if last_err() then return end
        i = i+1
    end
end

function run(arg)
    records = dns(arg['value'], {
        record='NS',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']

    i = 1
    while records[i] ~= nil do
        r = records[i][2]
        iter_ns(arg['value'], r['NS'])
        if last_err() then return end
        i = i+1
    end
end
