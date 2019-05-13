-- Description: Search for the same domain base on all TLDs
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: domains

function run(arg)
    local m = regex_find('^([^\\.]+)\\.', arg['value'])
    local base = m[2]

    -- TODO: we need a way to cache this
    -- TODO: .co.uk is missing
    local url = 'https://data.iana.org/TLD/tlds-alpha-by-domain.txt'

    local session = http_mksession()
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then
        return 'http error: ' .. resp['status']
    end

    local tlds = regex_find_all('([^\n]+)', resp['text'])

    for i=1, #tlds do
        local tld = tlds[i][1]:lower()

        if not tld:match('^#') then
            local domain = base .. '.' .. tld

            debug(domain)
            records = dns(domain, {
                record='NS',
            })
            if last_err() then
                clear_err()
            else
                if records['error'] == nil and records['answers'][1] then
                    debug(records)
                    db_add('domain', {
                        value=domain,
                    })
                end
            end
        end
    end
end
