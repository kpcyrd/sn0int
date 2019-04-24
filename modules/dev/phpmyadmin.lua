-- Description: Search for phpmyadmin
-- Version: 0.2.0
-- Source: urls
-- License: GPL-3.0

function run(arg)
    paths = {
        "phpmyadmin/index.php",
        "phpMyAdmin/index.php",
        "pmd/index.php",
        "pma/index.php",
        "PMA/index.php",
        "PMA2/index.php",
        "pmamy/index.php",
        "pmamy2/index.php",
        "mysql/index.php",
        "admin/index.php",
        "db/index.php",
        "dbadmin/index.php",
        "web/phpMyAdmin/index.php",
        "admin/pma/index.php",
        "admin/PMA/index.php",
        "admin/mysql/index.php",
        "admin/mysql2/index.php",
        "admin/phpmyadmin/index.php",
        "admin/phpMyAdmin/index.php",
        "admin/phpmyadmin2/index.php",
        "mysqladmin/index.php",
        "mysql-admin/index.php",
        "mysql_admin/index.php",
        "phpadmin/index.php",
        "phpAdmin/index.php",
        "phpmyadmin0/index.php",
        "phpmyadmin1/index.php",
        "phpmyadmin2/index.php",
        "phpMyAdmin-4.4.0/index.php",
        "myadmin/index.php",
        "myadmin2/index.php",
        "xampp/phpmyadmin/index.php",
        "phpMyadmin_bak/index.php",
        "www/phpMyAdmin/index.php",
        "tools/phpMyAdmin/index.php",
        "phpmyadmin-old/index.php",
        "phpMyAdminold/index.php",
        "phpMyAdmin.old/index.php",
        "pma-old/index.php",
        "claroline/phpMyAdmin/index.php",
        "typo3/phpmyadmin/index.php",
        "phpma/index.php",
        "phpmyadmin/phpmyadmin/index.php",
        "phpMyAdmin/phpMyAdmin/index.php",
        "phpMyAbmin/index.php",
        "phpMyAdmin__/index.php",
        "phpMyAdmin+++---/index.php",
        "v/index.php",
        "phpmyadm1n/index.php",
        "phpMyAdm1n/index.php",
        "shaAdmin/index.php",
        "phpMyadmi/index.php",
        "phpMyAdmion/index.php",
        "MyAdmin/index.php",
        "phpMyAdmin1/index.php",
        "phpMyAdmin123/index.php",
        "pwd/index.php",
        "phpMyAdmina/index.php",
        "program/index.php",
        "shopdb/index.php",
        "phppma/index.php",
        "phpmy/index.php",
        "mysql/admin/index.php",
        "mysql/dbadmin/index.php",
        "mysql/sqlmanager/index.php",
        "mysql/mysqlmanager/index.php",
        "wp-content/plugins/portable-phpmyadmin/wp-pma-mod/index.php",
    }

    session = http_mksession()

    for i=1, #paths do
        p = paths[i]
        url = url_join(arg['value'], p)
        debug(url)

        req = http_request(session, 'GET', url, {
            timeout=5000
        })
        reply = http_send(req)
        debug(reply)

        if last_err() then
            clear_err()
        else
            if reply['status'] == 200 then
                db_add('url', {
                    subdomain_id=arg['subdomain_id'],
                    value=url,
                    status=reply['status'],
                    body=reply['text'],
                })
            end
        end
    end
end
