-- Description: Extract exif data from images
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: images

function run(arg)
    exif = img_exif(arg['value'])
    if last_err() then return end
    debug(exif)

    db_update('image', arg, exif)
end
