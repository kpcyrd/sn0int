-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: subdomains

function run(arg)
    db_update('subdomain', arg, {
        resolvable=true,
    })
end
