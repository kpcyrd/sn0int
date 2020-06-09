-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

-- git log -s --format='%H %ci'

function run()
    while true do
        local x = stdin_readline()
        if x == nil then
            break
        end
        local m = regex_find('^(\\S+) (.+)', x)
        if m then
            time = strptime('%Y-%m-%d %T %z', m[3])
            time = sn0int_time_from(time)

            db_activity({
                topic='harness/sn0int-commit:dummy',
                time=time,
                uniq=m[2],
                content={},
            })
        end
    end
end
