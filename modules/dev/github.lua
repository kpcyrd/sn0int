-- Description: Collect data from github profiles
-- Version: 0.1.0
-- Source: accounts:github.com
-- License: GPL-3.0

function api_get(url)
    local req = http_request(session, 'GET', url, {})
    local resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'invalid status code' end

    local data = json_decode(resp['text'])
    if last_err() then return end
    return data
end

function scan4email(username)
    local url = 'https://api.github.com/users/' .. username .. '/repos'
    local repos = api_get(url)
    if last_err() then return end

    i = 1
    while i <= #repos do
        local repo = repos[i]
        debug(repo)
        local commits = api_get(repo['url'] .. '/commits')
        if last_err() then return end

        j = 1
        while j <= #commits do
            local commit = commits[j]
            debug(commit)

            if commit['author']['login'] == username then
                -- name = commit['commit']['author']['name']
                local email = commit['commit']['author']['email']
                return email
            end

            if commit['committer']['login'] == username then
                -- name = commit['commit']['committer']['name']
                local email = commit['commit']['committer']['email']
                return email
            end

            j = j+1
        end

        i = i+1
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
