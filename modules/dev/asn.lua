-- Description: Run a asn lookup for an ip address
-- Version: 0.1.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    lookup = asn_lookup(arg['value'])
    if last_err() then return end

    if arg['asn'] ~= lookup['asn'] or arg['as_org'] ~= lookup['as_org'] then
        db_update('ipaddr', arg, {
            asn=lookup['asn'],
            as_org=lookup['as_org'],
        })
    end
end
