-- Description: Query pgp keyserver for email addresses
-- Version: 0.2.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    session = http_mksession()

    --lookup_url = 'https://pgp.mit.edu/pks/lookup'
    lookup_url = 'https://sks-keyservers.net/pks/lookup'

    req = http_request(session, 'GET', lookup_url, {
        query={
            search=arg['value'],
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    links = html_select_list(resp['text'], 'a')
    for i=1, #links do
        href = links[i]['attrs']['href']

        if href:find('/pks/lookup%?op=get&search=') == 1 then
            url = url_join(lookup_url, href)

            req = http_request(session, 'GET', url, {})

            resp = http_send(req)
            -- TODO: do not abort script if one attempt fails
            if last_err() then return end
            if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

            pubkey = pgp_pubkey_armored(resp['text'])

            -- print(pubkey)

            -- TODO: ensure at least one email matches our target domain
            if pubkey['uids'] then
                for j=1, #pubkey['uids'] do
                    local m = regex_find("(.+) <([^< ]+@[^< ]+)>$", pubkey['uids'][j])
                    if m then
                        db_add('email', {
                            value=m[3],
                            displayname=m[2],
                        })
                    end
                end
            end
        end
    end
end
