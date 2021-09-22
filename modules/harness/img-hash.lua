-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0
-- Source: images

function run(arg)
    debug(arg)
    info(img_ahash(arg['value']))
    info(img_dhash(arg['value']))
    info(img_phash(arg['value']))
end
