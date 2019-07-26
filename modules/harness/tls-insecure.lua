-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    sock = sock_connect('expired.badssl.com', 443, {})
    if last_err() then return end

    tls = sock_upgrade_tls(sock, {
        sni_value='expired.badssl.com',
        disable_tls_verify=true,
    })
    if last_err() then return end

    info(tls)
end
