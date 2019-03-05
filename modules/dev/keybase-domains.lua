-- Description: Find keybase proofs for domains
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains

function run(arg)
    session = http_mksession()
    req = http_request(session, 'GET', 'https://keybase.io/_/api/1.0/user/lookup.json', {
        query={
            domain=arg['value'],
        }
    })
    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    x = json_decode(resp['text'])
    if last_err() then return end
    debug(x)

    if x['them'][1] == nil then return end
    them = x['them'][1]

    db_add('account', {
        service='keybase.io',
        username=them['basics']['username'],
    })
end
