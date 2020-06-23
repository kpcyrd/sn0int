-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains

function run(arg)
    for i=1, 20 do
        db_add('subdomain', {
            domain_id=arg['id'],
            value=http_mksession() .. '.' .. arg['value'],
        })
    end
end
