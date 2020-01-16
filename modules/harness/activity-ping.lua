-- Description: Log some dummy activity
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    local uniq = getopt('uniq')
    local topic = getopt('topic') or 'harness/activity-ping:dummy'

    if getopt('gps') then
        lat=1.23
        lon=4.56
        radius=100
    end

    while true do
        db_activity({
            topic=topic,
            time=sn0int_time(),
            uniq=uniq,
            latitude=lat,
            longitude=lon,
            radius=radius,
            content={
                msg='ohai',
            },
        })
        sleep(5)
    end
end
