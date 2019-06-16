-- Description: Create subdomain entries for apex domains
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains

function run(arg)
    db_add('subdomain', {
        domain_id=arg['id'],
        value=arg['value'],
    })
end
