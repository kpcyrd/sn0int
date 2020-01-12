-- Description: Log some dummy activity
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    local uniq = getopt('uniq')

    if getopt('gps') then
        lat=1.23
        lon=4.56
    end

    while true do
        db_activity({
            topic='harness/activity-ping:dummy',
            time=sn0int_time(),
            uniq=uniq,
            latitude=lat,
            longitude=lon,
            content={
                msg='ohai',
            },
        })
        sleep(5)
    end
end
