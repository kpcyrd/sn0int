-- Description: Scan collected images for nudity
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: images

function run(arg)
    local nudity = img_nudity(arg['value'])
    debug(nudity)
    db_update('image', arg, {
        nudity=nudity['score'],
    })
end
