-- Description: Query thunderbird autoconfig db for subdomains
-- Version: 0.2.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    session = http_mksession()

    -- check if an autoconfig exists without disclosing our target yet
    req = http_request(session, 'GET', 'https://autoconfig.thunderbird.net/v1.1/', {})
    resp = http_send(req)
    if last_err() then return end

    if resp['status'] ~= 200 then
        return 'index request failed'
    end

    if resp['text']:find(arg['value'], 1, true) == nil then
        debug('no autoconfig available')
        return
    end

    -- request config
    req = http_request(session, 'GET', 'https://autoconfig.thunderbird.net/v1.1/' .. arg['value'], {})
    resp = http_send(req)
    if last_err() then return end

    m = regex_find_all('<hostname>([^<]+)</hostname>', resp['text'])

    for i=1, #m do
        subdomain = m[i][2]

        domain = psl_domain_from_dns_name(subdomain)
        if last_err() then return end

        domain_id = db_select('domain', domain)
        if last_err() then return end

        db_add('subdomain', {
            domain_id=domain_id,
            value=subdomain,
        })
        if last_err() then return end
    end
end
