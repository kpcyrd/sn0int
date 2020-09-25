-- Description: Send a request to a hidden service
-- Version: 0.1.0
-- Stealth: passive
-- License: GPL-3.0

function run()
    local session = http_mksession()
    local req = http_request(session, 'GET', 'http://expyuzz4wqqyqhjn.onion/', {
        proxy='127.0.0.1:9050',
    })
    local r = http_fetch(req)
    if last_err() then return end

    local title = html_select(r['text'], 'title')
    if last_err() then return end
    info(title['html'])
end
