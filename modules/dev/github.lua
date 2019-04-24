-- Description: Collect data from github profiles
-- Version: 0.2.0
-- Source: accounts:github.com
-- License: GPL-3.0

function api_get(url)
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end
    -- TODO: set_error(?)
    if resp['status'] == 403 then return 'ratelimit exceeded' end
    if resp['status'] ~= 200 then return 'invalid status code' end

    local data = json_decode(resp['text'])
    if last_err() then return end
    return data
end

function import_gpg(url)
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end

    local key = pgp_pubkey_armored(resp['text'])
    if not key['uids'] then return end

    for i=1, #key['uids'] do
        local k = key['uids'][i]
        debug(k)
        local m = regex_find("(.+) <([^< ]+@[^< ]+)>$", k)
        if m then
            db_add('email', {
                value=m[3],
                displayname=m[2],
            })
        end
    end
end

function scan4email(username)
    local url = 'https://api.github.com/users/' .. username .. '/repos'
    local repos = api_get(url)
    if last_err() then return end

    -- XXX: 'https://api.github.com/users/' .. username .. '/events/public?page=0&per_page=100' is faster but less accurate

    for i=1, #repos do
        local repo = repos[i]
        debug(repo)
        local commits = api_get(repo['url'] .. '/commits')
        if last_err() then return end

        for j=1, #commits do
            local commit = commits[j]
            debug(commit)

            if commit['author'] and commit['author']['login'] == username then
                local name = commit['commit']['author']['name']
                local email = commit['commit']['author']['email']
                db_add('email', {
                    value=email,
                    displayname=name,
                })
                return email
            end

            if commit['committer'] and commit['committer']['login'] == username then
                local name = commit['commit']['committer']['name']
                local email = commit['commit']['committer']['email']
                db_add('email', {
                    value=email,
                    displayname=name,
                })
                return email
            end
        end
    end
end

function run(arg)
    session = http_mksession()
    local url = 'https://api.github.com/users/' .. arg['username']

    local data = api_get(url)
    if last_err() then return end
    debug(data)

    -- company = data['company']
    -- location = data['location']
    -- homepage = data['blog']

    url = 'https://github.com/' .. arg['username'] .. '.gpg'
    import_gpg(url)
    if last_err() then return end

    local email = data['email']
    if not email and not arg['email'] then
        email = scan4email(arg['username'])
        if last_err() then return end
    end

    db_update('account', arg, {
        url=data['html_url'],
        displayname=data['name'],
        email=email,
    })
end
