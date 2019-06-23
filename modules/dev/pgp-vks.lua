-- Description: Find alternative emails with vks lookups
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: emails

function run(arg)
    local keyserver = 'https://keys.openpgp.org'

    local url = keyserver .. '/vks/v1/by-email/' .. arg['value']
    local session = http_mksession()
    local req = http_request(session, 'GET', url, {})
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then return end

    local pubkey = pgp_pubkey_armored(r['text'])

    for i=1, #pubkey['uids'] do
        local uid = pubkey['uids'][i]
        debug(uid)

        local m = regex_find("(.+) <([^< ]+@[^< ]+)>$", uid)
        if m then
            local email = m[3]:lower()
            db_add('email', {
                value=email,
                displayname=m[2],
            })
        end
    end
end
