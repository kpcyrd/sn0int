-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: notifications

function run(arg)
    ratelimit_throttle('foo', 3, 10000)
    info('notication!: ' .. json_encode(arg))
end
