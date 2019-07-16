-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    info('adding domain')
    domain_id = db_add('domain', {
        value='example.com',
    })
    if last_err() then return end

    info('adding subdomain')
    subdomain_id = db_add('subdomain', {
        domain_id=domain_id,
        value='example.com',
    })
    if last_err() then return end

    info('adding ipaddr')
    ipaddr_id = db_add('ipaddr', {
        value='192.0.2.1',
    })
    if last_err() then return end

    info('adding device')
    device_id = db_add('device', {
        value='ff:ff:ff:ff:ff:ff',
    })
    if last_err() then return end

    info('adding network')
    network_id = db_add('network', {
        value='myssid',
    })
    if last_err() then return end

    info('adding email')
    email_id = db_add('email', {
        value='foo@example.com',
    })
    if last_err() then return end

    info('adding phonenumber')
    phonenumber_id = db_add('phonenumber', {
        value='+4912345678',
    })
    if last_err() then return end

    info('adding breach')
    breach_id = db_add('breach', {
        value='hack the planet',
    })
    if last_err() then return end

    info('adding account')
    account_id = db_add('account', {
        service='github.com',
        username='kpcyrd',
    })
    if last_err() then return end

    info('adding image')
    blob = create_blob('abc')
    image_id = db_add('image', {
        value=blob,
    })
    if last_err() then return end

    info('adding port')
    port_id = db_add('port', {
        ip_addr_id=ipaddr_id,
        ip_addr='192.0.2.1',
        port=443,
        protocol='tcp',
        status='open',
    })
    if last_err() then return end

    info('adding url')
    url_id = db_add('url', {
        subdomain_id=subdomain_id,
        value='https://www.example.com/a/b',
        body='<html></html>',
    })
    if last_err() then return end

    info('adding breach_email')
    db_add('breach-email', {
        breach_id=breach_id,
        email_id=email_id,
    })
    if last_err() then return end

    info('adding network_device')
    db_add('network-device', {
        network_id=network_id,
        device_id=device_id,
    })
    if last_err() then return end

    info('adding subdomain_ipaddr')
    db_add('subdomain-ipaddr', {
        subdomain_id=subdomain_id,
        ip_addr_id=ipaddr_id,
    })
    if last_err() then return end
end
