-- Description: Add an expiring domain
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    db_add_ttl('domain', {
        value='example.com',
    }, 30)
end
