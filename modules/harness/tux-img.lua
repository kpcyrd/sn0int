-- Description: Download an image
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    session = http_mksession()

    req = http_request(session, 'GET', 'https://www.kernel.org/theme/images/logos/tux.png', {
        into_blob=true,
    })
    r = http_fetch(req)
    if last_err() then return end

    debug(r)
    db_add('image', {
        value=r['blob'],
    })
end
