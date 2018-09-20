-- Description: Query subdomains to discovery ip addresses and verify the record is visible
-- Version: 0.1.0
-- Argument: subdomains

function run(arg)
    x = dns(arg['value'], 'A')
    info(json_encode(x))
end
