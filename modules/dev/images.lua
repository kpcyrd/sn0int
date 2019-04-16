-- Description: Parse image metadata
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: images

function run(arg)
    local img = img_load(arg['value'])
    db_update('image', arg, img)
end
