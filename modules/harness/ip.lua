-- Description: Show your ip
-- Version: 0.1.0
-- License: GPL-3.0

function get(url)
    req = http_request(session, 'GET', url, {})
    r = http_send(req)
    info(r['text'])
end

function run()
    session = http_mksession()
    get('https://icanhazip.com')
    get('https://icanhazptr.com')
end
