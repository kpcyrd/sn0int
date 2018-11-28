-- Description: Query subdomains to discovery ip addresses and verify the record is visible
-- Version: 0.1.0
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

    -- there is a bug in struct -> lua that causes tables to be zero indexed
    -- this checks if there's something at index 0 but uses index 1 if this is fixed
    i = 0
    if records[i] == nil then i = 1 end

    while records[i] ~= nil do
        r = records[i]
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
        end
        i = i+1
    end
end
