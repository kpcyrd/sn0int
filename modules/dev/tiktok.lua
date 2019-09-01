-- Description: Collect data from tiktok profiles
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:tiktok.com

function run(arg)
    local session = http_mksession()

    -- TODO: write a proper api client eventually

    local url = 'https://www.tiktok.com/@' .. arg['username']

    local req = http_request(session, 'GET', url, {})
    local r = http_fetch(req)
    if last_err() then return end

    local x = html_select(r['text'], '._user_header_nickName')
    local displayname = x['text']

    db_update('account', arg, {
        displayname=displayname,
        url=url,
    })
end
