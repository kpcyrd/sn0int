-- Description: Decode armored pgp key from stdin
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    local pubkey = stdin_read_to_end()
    pubkey = pgp_pubkey_armored(pubkey)
    info(pubkey)
end
