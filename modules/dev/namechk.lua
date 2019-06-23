-- Description: Find accounts by username with namechk.com
-- Version: 0.3.0
-- Source: accounts
-- License: GPL-3.0

function get_services(html)
    local divs = html_select_list(html, '.service')
    if last_err() then return end

    local services = {}

    for i=1, #divs do
        services[i] = divs[i]['attrs']['data-name']
    end

    return services
end

function run(arg)
    -- setup session
    local session = http_mksession()
    local req = http_request(session, 'GET', 'https://namechk.com/', {})
    local resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

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
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end
    debug(resp)

    local scan = json_decode(resp['text'])
    if last_err() then return end
    local scan_token = scan['valid']

    -- get results
    for i=1, #services do
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
        if last_err() then return end

        if resp['status'] == 200 then
            local acc = json_decode(resp['text'])
            if last_err() then return end
            debug(acc)

            if acc ~= nil and not acc['available'] and acc['status'] == 'unavailable' then
                db_add('account', {
                    service=services[i],
                    username=arg['username'],
                    url=acc['callback_url'],
                })
            end
        end
    end
end
