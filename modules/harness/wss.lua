-- Description: Create an encrypted websocket connection
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    -- local target = 'ws://echo.websocket.org' -- doesn't support proper ciphers
    local target = 'wss://rocket.events.ccc.de/sockjs/258/whi0yr1y/websocket'

    info('connecting to ' .. target)
    local sock = ws_connect(target, {})
    if last_err() then return end

    info('recieving 1/2')
    local msg = ws_recv_text(sock)
    if last_err() then return end

    if msg ~= 'o' then
        return 'recieve failed, got ' .. msg
    end

    info('recieving 2/2')
    local msg = ws_recv_text(sock)
    if last_err() then return end

    if msg ~= 'a["{\\"server_id\\":\\"0\\"}"]' then
        return 'recieve failed, got ' .. msg
    end

    info('handshake succeeded')
end
