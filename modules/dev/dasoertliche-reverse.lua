-- Description: Run phonenumber reverse lookups on dasoertliche.de
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: phonenumbers

function run(arg)
    session = http_mksession()
    req = http_request(session, 'GET', 'https://www.dasoertliche.de/Controller', {
        query={
            form_name='search_inv',
            ph=arg['value'],
        }
    })
    r = http_send(req)
    if last_err() then return end
    if r['status'] ~= 200 then return 'http error: ' .. r['status'] end

    x = html_select(r['text'], '#entry_1 a')
    if not x then
        -- no results
        return clear_err()
    end

    -- strip &nbsp;
    m = regex_find('[^\\xa0]+', x['text'])

    if m then
        db_update('phonenumber', arg, {
            caller_name=m[1],
        })
    end
end
