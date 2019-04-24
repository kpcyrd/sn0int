-- Description: Run reverse dns lookups
-- Version: 0.2.0
-- Source: ipaddrs
-- License: GPL-3.0

function run(arg)
    if arg['family'] == '4' then
        m = regex_find('^(\\d+)\\.(\\d+)\\.(\\d+)\\.(\\d+)$', arg['value'])

        q = m[5] .. '.' .. m[4] .. '.' .. m[3] .. '.' .. m[2] .. '.in-addr.arpa'
        debug('Resolving: ' .. q)

        records = dns(q, {
            record='PTR',
        })
        if last_err() then return end
        if records['error'] ~= nil then return end
        records = records['answers']

        for i=1, #records do
            r = records[i][2]
            if r['PTR'] then
                db_update('ipaddr', arg, {
                    reverse_dns=r['PTR'],
                })
                if last_err() then return end
            end
        end
    end
end
