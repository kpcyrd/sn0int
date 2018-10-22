-- Description: Query certificate transparency logs to discover subdomains
-- Version: 0.1.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    x = geoip_lookup(arg['value'])
    print('')
    print(x)
end
