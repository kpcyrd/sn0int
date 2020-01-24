-- Description: Test various tls functions
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    info('sending https request to google.com')
    session = http_mksession()
    req = http_request(session, 'GET', 'https://google.com/', {})
    r = http_send(req)
    if last_err() then return end
    debug(r)

    info('creating tls socket to google.com')
    sock = sock_connect('google.com', 443, {
        tls=true,
    })
    if last_err() then return end
    debug(sock)

    info('creating socket to google.com, wrapping afterwards')
    sock = sock_connect('google.com', 443, {})
    if last_err() then return end
    tls = sock_upgrade_tls(sock, {
        sni_value='google.com',
    })
    if last_err() then return end
    debug(sock)
    debug(tls)
end
