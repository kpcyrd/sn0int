-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    ip_addr = '192.168.1.2'
    ip_addr_id = db_add('ipaddr', {
        value=ip_addr,
    })
    db_add('port', {
        ip_addr_id=ip_addr_id,
        ip_addr=ip_addr,
        protocol='tcp',
        port=4444,
        status='open',
    })
end
