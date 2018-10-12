-- Description: Query pgp keyserver for email addresses
-- Version: 0.1.0
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
    i = 1
    while i <= #links do
        href = links[i]['attrs']['href']

        if href:find('/pks/lookup%?op=get&search=') == 1 then
            url = url_join(lookup_url, href)

            req = http_request(session, 'GET', url, {})

            resp = http_send(req)
            -- TODO: do not abort script if one attempt fails
            if last_err() then return end
            if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

            pubkey = pgp_pubkey_armored(resp['text'])

            print(pubkey)

            -- TODO: ensure at least one email matches our target domain
            if pubkey['uids'] then
                j = 1
                while j <= #pubkey['uids'] do
                    print(pubkey['uids'][j])
                    j = j+1
                end
            end
        end

        i = i+1
    end
end
