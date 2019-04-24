-- Description: Query certificate transparency logs to discover subdomains
-- Version: 0.5.0
-- Source: domains
-- License: GPL-3.0

function each_name(name)
    local domain_id, psl_domain

    if seen[name] == 1 then
        return
    end
    seen[name] = 1
    debug(name)

    if name:find('*.') == 1 then
        -- ignore wildcard domains
        return
    end

    -- the cert might be valid for subdomains that do not belong to the
    -- domain we started with
    psl_domain = psl_domain_from_dns_name(name)
    domain_id = domains[psl_domain]
    if domain_id == nil then
        if any_domain then
            -- unknown domains should be added to database
            domain_id = db_add('domain', {
                value=psl_domain,
            })
        else
            -- only use domains that are already in scope
            domain_id = db_select('domain', psl_domain)
        end

        -- if we didn't get a valid id, skip
        if domain_id == nil then
            return
        end

        domains[psl_domain] = domain_id
    end

    db_add('subdomain', {
        domain_id=domain_id,
        value=name,
    })
end

function run(arg)
    full = getopt('full') ~= nil
    any_domain = getopt('any-domain') ~= nil

    domains = {}
    domains[arg['value']] = arg['id']

    session = http_mksession()
    req = http_request(session, 'GET', 'https://crt.sh/', {
        query={
            q='%.' .. arg['value'],
            output='json'
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    certs = json_decode(resp['text'])
    if last_err() then return end

    seen = {}

    for i=1, #certs do
        c = certs[i]
        debug(c)

        if full then
            -- fetch certificate
            id = c['min_cert_id']
            req = http_request(session, 'GET', 'https://crt.sh/', {
                query={
                    d=id .. '', -- TODO: find nicer way for tostring
                }
            })
            resp = http_send(req)
            if last_err() then return end
            if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

            -- iterate over all valid names
            crt = x509_parse_pem(resp['text'])
            if last_err() then return end
            names = crt['valid_names']

            for j=1, #names do
                each_name(names[j])
            end
        else
            each_name(c['name_value'])
        end
    end
end
