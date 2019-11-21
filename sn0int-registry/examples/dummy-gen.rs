#[macro_use] extern crate failure;

use sn0int_common::metadata::Metadata;
use semver::Version;
use sn0int_registry::db::wait_for_db;
use diesel::PgConnection;
use dotenv::dotenv;
use sn0int_registry::models::*;
use diesel::Connection;
use std::env;
use sn0int_registry::errors::*;

fn insert(connection: &PgConnection, user: &str, name: &str, code: &str) -> Result<()> {
    let metadata = code.parse::<Metadata>()
        .context("Failed to parse module metadata")?;

    let version = metadata.version.clone();
    Version::parse(&version)
        .context("Version is invalid")?;

    connection.transaction::<_, Error, _>(|| {
        let module = Module::update_or_create(&user, &name, &metadata, &connection)
            .context("Failed to write module metadata")?;

        match Release::try_find(module.id, &version, &connection)? {
            Some(release) => {
                // if the code is identical, pretend we published the version
                if release.code != code {
                    bail!("Version number already in use")
                }
            },
            None => module.add_version(&version, &code, &connection)
                .context("Failed to add release")?,
        }
        println!("[+] added {}/{}", module.author, module.name);

        Ok(())
    })
}


fn run() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;
    let connection = wait_for_db(&database_url, 5)?;

    insert(&connection, "kpcyrd", "ctlogs", r#"-- Description: Query certificate transparency logs to discover subdomains
-- Version: 0.5.0
-- Source: domains
-- License: GPL-3.0

function run(arg)
    full = getopt('full') ~= nil
    any_domain = getopt('any-domain') ~= nil

    domains = {}
    domains[arg['value']] = arg['id']

    session = http_mksession()
    req = http_request(session, 'GET', 'https://crt.sh/', {
        query={
            q='%.' .. arg['value'],
            output='json'
        }
    })

    resp = http_send(req)
    if last_err() then return end
    if resp['status'] ~= 200 then return 'http error: ' .. resp['status'] end

    certs = json_decode(resp['text'])
    if last_err() then return end
"#)?;

    insert(&connection, "dead", "module", r#"-- Description: TODO your description here
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    -- TODO your code here
end
"#)?;

    let dead = Module::find("dead", "module", &connection)?;
    dead.redirect(&"kpcyrd/ctlogs".parse().unwrap(), &connection)?;

    Ok(())
}

fn main() {
    env_logger::init();

    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
