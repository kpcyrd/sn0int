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
        local pkt = mqtt_recv_text(sock)
        if last_err() then return end
        info(pkt)
    end
end
