-- Description: Find IPs using certificates for target subdomains
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: subdomains
-- Keyring-Access: shodan

function contains(list, search)
    for i=1, #list do
        if list[i] == search then
            return true
        end
    end
    return false
end

function run(arg)
    local key = keyring('shodan')[1]
    if not key then
        return 'Missing required shodan access key'
    end
    local access_key = key['access_key']

    local session = http_mksession()
    local req = http_request(session, 'GET', 'https://api.shodan.io/shodan/host/search', {
        query={
            key=access_key,
            query='ssl:' .. arg['value'],
        }
    })
    local data = http_fetch_json(req)
    if last_err() then return end

    for i=1, #data['matches'] do
        local host = data['matches'][i]
        -- debug(host)

        local crt = x509_parse_pem(host['ssl']['chain'][1])
        if last_err() then return end

        ipaddr_id = db_add('ipaddr', {
            value=host['ip_str'],
        })
        if last_err() then return end

        if contains(crt['valid_names'], arg['value']) then
            db_add('subdomain-ipaddr', {
                subdomain_id=arg['id'],
                ip_addr_id=ipaddr_id,
            })
            if last_err() then return end
        end
    end
end
