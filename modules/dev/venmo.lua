-- Description: Fetch data from venmo profiles
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: accounts:venmo.com

-- based on https://github.com/mportatoes/venemy

function download_blob(session, url)
    while true do
        local req = http_request(session, 'GET', url, {
            into_blob=true,
        })
        local r = http_send(req)
        if last_err() then return end

        if r['status'] == 200 then
            return r
        elseif r['headers']['location'] then
            url = r['headers']['location']
        else
            return 'http error: ' .. r['status']
        end
    end
end

function run(arg)
    local session = http_mksession()

    local url = 'https://api.venmo.com/v1/users/' .. arg['username']
    local req = http_request(session, 'GET', url, {})
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then
        return 'http error: ' .. r['status']
    end

    local data = json_decode(r['text'])
    if last_err() then return end
    data = data['data']
    debug(data)

    if data['profile_picture_url'] then
        local url = data['profile_picture_url']

        -- enhance facebook images
        local m = regex_find('https://graph\\.facebook\\.com/v2\\.10/(\\d+)/picture\\?type=square', url)
        if m then
            url = 'https://graph.facebook.com/v2.10/' .. m[2] .. '/picture?type=large'
            db_add('account', {
                service='facebook.com',
                username=''..m[2],
            })
        end

        r = download_blob(session, url)
        if last_err() then return end

        -- TODO: link image to profile
        db_add('image', {
            value=r['blob'],
        })
    end

    db_update('account', arg, {
        displayname=data['display_name'],
        url='https://venmo.com/' .. arg['value'],
    })
end
