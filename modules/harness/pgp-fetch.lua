-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    session = http_mksession()
    url = 'https://openpgpkey.archlinux.org/.well-known/openpgpkey/archlinux.org/hu/in9mwr4s84x7gm51851h343n3at1x61g?l=anthraxx'

    req = http_request(session, 'GET', url, {})
    r = http_fetch(req)
    -- debug(r)
    k = pgp_pubkey(r['text'])
    info(k)

    req = http_request(session, 'GET', url, {
        binary=true,
    })
    r = http_fetch(req)
    -- debug(r)
    k = pgp_pubkey(r['binary'])
    info(k)
end
