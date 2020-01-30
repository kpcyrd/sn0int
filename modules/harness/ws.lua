-- Description: Create an echo websocket connection
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    local target = 'ws://echo.websocket.org'

    info('connecting to ' .. target)
    local sock = ws_connect(target, {})
    if last_err() then return end

    info('sending')
    ws_send_text(sock, 'ohai wurld')
    if last_err() then return end

    info('recieving')
    local msg = ws_recv_text(sock)
    if last_err() then return end

    if msg ~= 'ohai wurld' then
        return 'echo failed, got: ' .. msg
    end
end
