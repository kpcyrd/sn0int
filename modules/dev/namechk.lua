-- Description: Find accounts by username with namechk.com
-- Version: 0.1.0
-- Source: accounts
-- License: GPL-3.0

function get_services(html)
    local divs = html_select_list(html, '.service')

    local services = {}

    local i = 1
    while i <= #divs do
        services[i] = divs[i]['attrs']['data-name']
        i = i+1
    end

    return services
end

function run(arg)
    -- setup session
    local session = http_mksession()
    local req = http_request(session, 'GET', 'https://namechk.com/', {})
    local resp = http_send(req)

    local token = html_select(resp['text'], 'input[name="authenticity_token"]')
    local auth_token = token['attrs']['value']

        local headers = {}
        headers['X-CSRF-Token'] = authenticity_token

    local services = get_services(resp['text'])
    debug({
        auth_token=auth_token,
        services=services,
    })

    -- trigger the scan
    local req = http_request(session, 'POST', 'https://namechk.com/', {
            headers=headers,
        form={
            authenticity_token=auth_token,
            q=arg['username'],
        }
    })
    local resp = http_send(req)
    debug(resp)
    local scan = json_decode(resp['text'])
    local scan_token = scan['valid']

    -- get results
    local i = 1
    while i <= #services do
        debug(services[i])

        local req = http_request(session, 'POST', 'https://namechk.com/services/check', {
            headers=headers,
            form={
                token=scan_token,
                fat=auth_token,
                service=services[i],
            }
        })
        local resp = http_send(req)

        local acc = json_decode(resp['text'])
        debug(acc)

        if acc ~= nil and not acc['available'] and acc['status'] == 'unavailable' then
            db_add('account', {
                service=services[i],
                username=arg['username'],
                url=acc['callback_url'],
            })
        end

        i = i+1
    end
end
