-- Description: Run phonenumber reverse lookups on dastelefonbuch.de
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: phonenumbers

function run(arg)
    session = http_mksession()

    -- TODO: fix url encoding for +
    query = arg['value']
    query = query:gsub('+', '%%2B')
    url = 'https://www.dastelefonbuch.de/R%C3%BCckw%C3%A4rts-Suche/' .. query

    req = http_request(session, 'GET', url, {})
    r = http_send(req)
    if last_err() then return end

    -- no data available
    if r['status'] == 410 then return end
    -- an error occured
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end

    x = html_select(r['text'], '#entry_1 div')
    name = x['attrs']['title']

    db_update('phonenumber', arg, {
        caller_name=name,
    })
end
