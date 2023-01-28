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
        -- read the next mqtt packet
        local pkt = mqtt_recv(sock)
        if last_err() then return end

        local text
        if pkt then
            -- attempt to utf8 decode the body if there was a pkt
            text = utf8_decode(pkt['body'])
            if last_err() then clear_err() end
        end

        info({pkt=pkt, text=text})
    end
end
