-- Description: Scrape known http responses for urls
-- Version: 0.1.0
-- Source: urls
-- License: GPL-3.0

function entry(target, parent, href)
    -- TODO: parse mailto:foo@example.com?subject=asdf
    -- TODO: parse tel:+4912345
    -- TODO: allow discovering 3rd-party domains
    -- TODO: maybe record urls as well

    local psl, parts, url, host

    if href == nil then
        return
    end

    url = url_join(parent, href)
    if url:match('^https?://') == nil then
        return
    end

    parts = url_parse(url)
    if last_err() then return end
    host = parts['host']
    psl = psl_domain_from_dns_name(host)

    if psl ~= target then
        -- TODO: this doesn't match the current target, but might match a different target in scope
        -- if we can check an entry exists in the db we could make this more intelligent
        return
    end

    domain_id = db_add('domain', {
        value=psl,
    })
    db_add('subdomain', {
        domain_id=domain_id,
        value=host,
    })
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

    -- get public suffix
    url = url_parse(arg['value'])
    if last_err() then return end
    psl = psl_domain_from_dns_name(url['host'])

    -- process html links
    i = 1
    while i <= #links do
        href = links[i]['attrs']['href']

        entry(psl, arg['value'], href)

        i = i+1
    end
end
