-- Description: Request access to keyring
-- Version: 0.1.0
-- Keyring-Access: twilio
-- License: GPL-3.0

function run(arg)
    keys = keyring('twilio')
    debug(keys)
end
