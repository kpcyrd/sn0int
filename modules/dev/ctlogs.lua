-- Description: Query certificate transparency logs to discover subdomains
-- Version: 0.1.0
-- Source: domains

function run(arg)
    session = http_mksession()

    -- TODO: example.com needs to be dynamic
    req = http_request(session, 'GET', 'https://crt.sh/', {
        query={
            q='%.' .. arg['value'],
            output='json'
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    certs = json_decode_stream(resp['text'])
    if last_err() then return end

    seen = {}

    i = 1
    while i <= #certs do
        c = certs[i]
        -- print(c)

        name = c['name_value']

        if name:find("*.") == 1 then
            -- ignore wildcard domains
            seen[name] = 1
        end

        if seen[name] == nil then
            -- info(name)
            db_add('subdomain', {
                domain_id=arg['id'],
                value=name,
            })
            seen[name] = 1
        end

        i = i+1
    end
end
