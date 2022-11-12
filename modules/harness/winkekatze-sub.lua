-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    local sock = mqtt_connect('mqtt://mqtt.winkekatze24.de', {
        read_timeout=10,
    })
    if last_err() then return end
    mqtt_subscribe(sock, '#', 0)

    while true do
        local pkt = mqtt_recv(sock)
        if last_err() then return end
        local log = {pkt=pkt}
        if pkt then
            log['text'] = utf8_decode(pkt['body'])
            if last_err() then clear_err() end
        end
        info(log)
    end
end
