-- Description: Parse arp-scan output
-- Version: 0.1.0
-- License: GPL-3.0

-- sudo arp-scan -qglI wlp3s0

function run()
    while true do
        x = stdin_readline()
        if x == nil then
            break
        end

        m = regex_find('(.+)\t(.+)', x)
        if m ~= nil then
            ip = m[2]
            mac = m[3]
            info({ip, mac})
        end
    end
end
