-- Description: Collect accounts and emails from keybase accounts
-- Version: 0.2.0
-- License: GPL-3.0
-- Source: accounts:keybase.io

function extract_mails(pubkey)
    for j=1, #pubkey['uids'] do
        local m = regex_find("(.+) <([^< ]+@[^< ]+)>$", pubkey['uids'][j])
        if m then
            db_add('email', {
                value=m[3],
                displayname=m[2],
            })
        end
    end
end

function add_domain(dns)
    local domain = psl_domain_from_dns_name(dns)
    if last_err() then return end

    local domain_id = db_add('domain', {
        value=domain,
    })
    if last_err() then return end
    if domain_id == nil then return end

    if domain ~= dns then
        db_add('subdomain', {
            domain_id=domain_id,
            value=dns,
        })
    end
end

function run(arg)
    session = http_mksession()
    req = http_request(session, 'GET', 'https://keybase.io/_/api/1.0/user/lookup.json', {
        query={
            usernames=arg['username'],
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    x = json_decode(resp['text'])
    if last_err() then return end
    debug(x)

    if x['them'][1] == nil then return end
    them = x['them'][1]

    -- update keybase profile
    db_update('account', arg, {
        displayname=them['profile']['full_name'],
        url='https://keybase.io/'..arg['username'],
    })

    -- collect emails
    pubkey = pgp_pubkey_armored(them['public_keys']['primary']['bundle'])
    debug(pubkey)
    extract_mails(pubkey)

    -- collect profiles
    profiles = them['proofs_summary']['all']

    for i=1, #profiles do
        profile = profiles[i]
        debug(profile)

        if
            profile['proof_type'] == 'generic_web_site' or
            profile['proof_type'] == 'dns'
        then
            add_domain(profile['nametag'])
        else
            db_add('account', {
                service=profile['proof_type'],
                username=profile['nametag'],
                url=profile['service_url'],
            })
        end
    end
end
