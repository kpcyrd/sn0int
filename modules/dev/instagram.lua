-- Description: Collect data from instagram profiles
-- Version: 0.2.0
-- Source: accounts:instagram.com
-- License: GPL-3.0

PAGE_SIZE = 50

function get_shared_data(html)
    local s = html_select_list(html, 'script')

    for i=1, #s do
        local m = regex_find('^window\\._sharedData = (.+);$', s[i]['text'])
        if m then
            return json_decode(m[2])
        end
    end
end

function sign_request(rhx, json_params)
    local magic = rhx .. ':' .. json_params
    local x_instagram_gis = hex(md5(magic))
    return x_instagram_gis
end

function download_image(node)
    local url = node['display_url']
    debug(url)

    local req = http_request(session, 'GET', url, {
        into_blob=true,
    })
    local r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end

    db_add('image', {
        value=r['blob'],
    })
end

function pull_graphql(page)
    local end_cursor = page['page_info']['end_cursor']

    for i=1, #page['edges'] do
        -- shortcode = page['edges'][i]['shortcode']
        local node = page['edges'][i]['node']
        node['thumbnail_resources'] = nil
        node['media_preview'] = nil
        -- debug(node)

        -- if node['__typename'] == 'GraphImage'

        -- node['dimensions']['height']
        -- node['dimensions']['width']
        -- ^ not sure how to get that picture

        -- node['taken_at_timestamp']
        -- location = node['location']

        local err = download_image(node)
        if last_err() then return end
        if err ~= nil then return err end

        todo_posts = todo_posts -1
        debug('posts left: ' .. todo_posts .. '/' .. total_posts)
    end

    if page['page_info']['has_next_page'] then
        debug('requesting next page=' .. end_cursor)

        variables = json_encode({
            id=user['id'],
            first=PAGE_SIZE,
            after=end_cursor
        })

        local headers = {}
        headers['X-Instagram-GIS'] = sign_request(rhx_gis, variables)

        local req = http_request(session, 'GET', 'https://www.instagram.com/graphql/query/', {
            query={
                query_hash='42323d64886122307be10013ad2dcc44',
                variables=variables,
            },
            headers=headers,
        })
        r = http_send(req)
        if last_err() then return end
        if r['status'] ~= 200 then return 'http error: ' .. r['status'] end

        x = json_decode(r['text'])
        if last_err() then return end
        return pull_graphql(x['data']['user']['edge_owner_to_timeline_media'])
    end
end

function run(arg)
    session = http_mksession()
    local url = 'https://www.instagram.com/' .. arg['username'] .. '/'
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'invalid status code' end
    local html = resp['text']

    local ld = html_select(html, 'script[type="application/ld+json"]')
    if last_err() then return end

    local ld = json_decode(ld['text'])
    if last_err() then return end
    --debug(ld)

    if ld['email'] then
        db_add('email', {
            value=ld['email'],
        })
    end

    -- homepage=ld['url']

    db_update('account', arg, {
        displayname=ld['name'],
        email=ld['email'],
        url=url,
    })

    -- download images
    local sd = get_shared_data(html)
    if last_err() then return end
    -- debug(sd)

    rhx_gis = sd['rhx_gis']
    user = sd['entry_data']['ProfilePage'][1]['graphql']['user']

    -- user['full_name']
    -- user['id']
    -- user['is_business_account']
    -- user['is_private']
    -- user['is_verified']
    -- user['has_blocked_viewer']
    -- user['connected_fb_page']
    -- user['country_block']

    local page = user['edge_owner_to_timeline_media']
    total_posts = page['count']
    todo_posts = total_posts

    -- TODO: fast-update abort if image has been downloaded already
    return pull_graphql(page)
end
