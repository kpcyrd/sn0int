-- Description: Import subdomains from stdin
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    -- echo 'www.example.com' | sn0int run -vvf --stdin modules/harness/import-subdomains.lua

    while true do
        local line = stdin_readline()
        if line == nil then
            break
        end
        -- strip newline
        local m = regex_find('.+', line)
        if m then
            local subdomain = m[1]

            local domain = psl_domain_from_dns_name(subdomain)
            local domain_id = db_add('domain', {
                value=domain,
            })

            db_add('subdomain', {
                domain_id=domain_id,
                value=subdomain,
            })
        end
    end
end
