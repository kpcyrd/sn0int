-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    sock = sock_connect('badssl.com', 443, {})
    if last_err() then return end

    tls = sock_upgrade_tls(sock, {
        sni_value='badssl.com',
    })
    if last_err() then return end

    info(tls)

    info(x509_parse_pem(tls['cert']))
end
