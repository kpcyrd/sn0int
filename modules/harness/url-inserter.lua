-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    info('preparing')
    domain_id = db_add('domain', {
        value='example.com',
    })
    subdomain_id = db_add('subdomain', {
        domain_id=domain_id,
        value='example.com',
    })

    info('inserting')
    url1 = db_add('url', {
        subdomain_id=subdomain_id,
        value='https://example.com',
    })
    url2 = db_add('url', {
        subdomain_id=subdomain_id,
        value='https://example.com/ohai',
        body='ohai',
    })
    url3 = db_add('url', {
        subdomain_id=subdomain_id,
        value='https://example.com/world',
        body={0x77, 0x6f, 0x72, 0x6c, 0x64},
    })

    info('updating')
    db_update('url', {
        id=url1,
        subdomain_id=subdomain_id,
        value='https://example.com',
        path='/',
        unscoped=false,
    }, {
    })
    db_update('url', {
        id=url2,
        subdomain_id=subdomain_id,
        value='https://example.com/ohai',
        path='/ohai',
        body='ohai',
        unscoped=false,
    }, {
    })
    db_update('url', {
        id=url3,
        subdomain_id=subdomain_id,
        value='https://example.com/world',
        path='/world',
        body={0x77, 0x6f, 0x72, 0x6c, 0x64},
        unscoped=false,
    }, {
    })
end
