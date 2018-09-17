-- Description: Query certificate transparency logs to discover subdomains
-- Version: 0.1.0

function run()
    session = http_mksession()

    -- TODO: example.com needs to be dynamic
    req = http_request(session, 'GET', 'https://crt.sh/', {
        query={
            q='%.' .. 'example.com',
            output='json'
        }
    })

    resp = http_send(req)
    if last_err() then return end

    certs = json_decode_stream(resp['text'])
    if last_err() then return end

    seen = {}

    i = 1
    while i <= #certs do
        c = certs[i]
        -- print(c)

        name = c['name_value']
        if seen[name] == nil then
            info(name)
            seen[name] = 1
        end

        i = i+1
    end
end
