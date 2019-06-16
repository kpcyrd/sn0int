-- Description: Search for domains pointing to ip
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: ipaddrs

-- XXX: the api ignores subdomains
function run(arg)
    local session = http_mksession()
    local req = http_request(session, 'GET', 'https://viewdns.info/reverseip/', {
        query={
            host=arg['value'],
            t='1',
        }
    })
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then
        return 'http error: ' .. r['status']
    end
    local html = r['text']

    local domains = html_select_list(html, '#null table td:first-child')
    for i=2, #domains do
        local domain = domains[i]['text']
        db_add('domain', {
            value=domain,
        })
    end
end
