-- Description: Request subdomains from spyse.com
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains
-- Keyring-Access: spyse

function run(arg)
    local key = keyring('spyse')[1]
    if not key then
        return 'Missing required spyse access key'
    end
    local access_key = key['access_key']

    local session = http_mksession()

    local page=1
    while true do
        local req = http_request(session, 'GET', 'https://api.spyse.com/v1/subdomains', {
            query={
                api_token=key['access_key'],
                domain=arg['value'],
                page=''..page,
            }
        })
        debug('sending request for page #' .. page)
        local r = http_send(req)
        if last_err() then return end
        if r['status'] ~= 200 then return 'http error: ' .. r['status'] end

        local data = json_decode(r['text'])
        if last_err() then return end

        if #data['records'] == 0 then
            break
        end

        for i=1, #data['records'] do
            local record = data['records'][i]
            local subdomain = record['domain']
            -- info(record['ip']['ip'])
            db_add('subdomain', {
                domain_id=arg['id'],
                value=subdomain,
            })
        end

        page = page+1
    end
end
