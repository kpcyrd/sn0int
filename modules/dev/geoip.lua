-- Description: Run a geoip lookup for an ip address
-- Version: 0.1.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    lookup = geoip_lookup(arg['value'])
    if last_err() then return end

    fields = {
        'continent',
        'continent_code',
        'country',
        'country_code',
        'city',
        'latitude',
        'longitude',
    }

    update = {}
    updated = false

    i = 1
    while i <= #fields do
        f = fields[i]

        if lookup[f] ~= arg[f] then
            update[f] = lookup[f]
            updated = true
        end

        i = i+1
    end

    if updated then
        db_update('ipaddr', arg, update)
    end
end
