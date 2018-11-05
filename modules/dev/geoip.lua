-- Description: Run a geoip lookup for an ip address
-- Version: 0.1.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    lookup = geoip_lookup(arg['value'])
    if last_err() then return end
    db_update('ipaddr', arg, lookup)
end
