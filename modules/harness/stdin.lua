-- Description: Read from stdin
-- Version: 0.1.0
-- Stealth: offline
-- License: GPL-3.0

function run()
    while true do
        x = stdin_readline()
        if x == nil then
            break
        end
        info(x)
    end
end
