-- Description: Verify email address by asking the smtp server
-- Version: 0.2.0
-- Source: emails
-- License: GPL-3.0

function find_mx(domain)
    local records, i, r

    records = dns(domain, {
        record='MX',
    })
    if last_err() then return end
    if records['error'] ~= nil then return end
    records = records['answers']
    -- debug(records)

    for i=1, #records do
        r = records[i][2]['MX']
        if r then
            debug('mx: ' .. r[2])
            return r[2]
        end
    end
end

function run(arg)
    -- extract domain
    domain = arg['value']:match('@(.*)')
    if doman ~= nil then
        -- malformed domain
        return
    end

    -- mx lookup
    mx = find_mx(domain)
    if last_err() then return end
    if not mx then return end

    -- create connection
    c = sock_connect(mx, 25, {})
    if last_err() then return end

    l = sock_recvline(c)
    if last_err() then return end
    debug(l)

    -- send hello
    sock_sendline(c, 'ehlo localhost')
    if last_err() then return end

    l = sock_recvline_regex(c, '^250 ')
    if last_err() then return end
    debug(l)

    -- send email
    sock_sendline(c, 'mail from:<root@localhost>')
    if last_err() then return end

    l = sock_recvline(c)
    if last_err() then return end
    debug(l)

    -- send rcpt
    sock_sendline(c, 'rcpt to:<' .. arg['value'] .. '>')
    if last_err() then return end

    l = sock_recvline(c)
    if last_err() then return end
    debug(l)

    -- check status
    verified = nil
    if l:match('^2') then
        debug('email is valid')
        verified = true
    elseif l:match('^5') then
        debug('email is invalid')
        verified = false
    elseif l:match('^4') then
        debug('unknown status, temporary delivery failure')
    end

    if verified ~= nil then
        db_update('email', arg, {
            valid=verified,
        })
    end
end
