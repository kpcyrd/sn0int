-- Description: Query subdomains to discovery ip addresses and verify the record is visible
-- Version: 0.3.0
-- Source: subdomains
-- License: GPL-3.0

function run(arg)
    records = dns(arg['value'], 'A')
    if last_err() then return end

    -- update subdomain
    resolvable = records['error'] == nil
    if arg['resolvable'] ~= resolvable then
        -- TODO: pass arg to function as well
        db_update('subdomain', arg, {
            resolvable=resolvable
        })
    end

    if not resolvable then
        return
    end

    records = records['answers']

    for i=1, #records do
        r = records[i][2]
        if r['A'] ~= nil then
            ipaddr_id = db_add('ipaddr', {
                family='4',
                value=r['A'],
            })
            if last_err() then return end

            db_add('subdomain-ipaddr', {
                subdomain_id=arg['id'],
                ip_addr_id=ipaddr_id,
            })
            if last_err() then return end
        end
    end
end
