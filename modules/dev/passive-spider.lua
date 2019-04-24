-- Description: Scrape known http responses for urls
-- Version: 0.2.0
-- Source: urls
-- License: GPL-3.0

function entry(parent, href)
    -- TODO: parse mailto:foo@example.com?subject=asdf
    -- TODO: parse tel:+4912345
    -- TODO: allow discovering 3rd-party domains
    -- TODO: maybe record urls as well

    local psl, parts, url, host

    if href == nil then
        return
    end

    url = url_join(parent, href)
    if last_err() then return clear_err() end
    if url:match('^https?://') == nil then
        return
    end

    parts = url_parse(url)
    if last_err() then return end
    host = parts['host']
    psl = psl_domain_from_dns_name(host)


    domain_id = db_select('domain', psl)
    if domain_id ~= nil then
        db_add('subdomain', {
            domain_id=domain_id,
            value=host,
        })
    end
end

function run(arg)
    if arg['body'] == nil or #arg['body'] == 0 then
        return
    end

    body = utf8_decode(arg['body'])
    if last_err() then return end

    links = html_select_list(body, 'a')
    if last_err() then return end

    if #links == 0 then
        return
    end

    -- process html links
    for i=1, #links do
        href = links[i]['attrs']['href']

        entry(arg['value'], href)
    end
end
