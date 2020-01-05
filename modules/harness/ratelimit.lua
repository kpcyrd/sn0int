-- Description: Run script with a global ratelimit
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    -- this shouldn't complete in less than 5 seconds
    for i=1, 20 do
        ratelimit_throttle('foo', 4, 1000)
        info(sn0int_time())
    end
end
