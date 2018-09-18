use errors::*;

use diesel;
use diesel::prelude::*;
use models::*;
use schema::*;
use paths;
use migrations;
use worker;


pub struct Database {
    name: String,
    db: SqliteConnection,
}

impl Database {
    pub fn establish<I: Into<String>>(name: I) -> Result<Database> {
        // TODO: enforce safe name for database
        let name = name.into();

        let path = paths::data_dir()?.join(name.clone() + ".db");
        let path = path.into_os_string().into_string()
            .map_err(|_| format_err!("Failed to convert db path to utf-8"))?;

        let db = worker::spawn_fn("Connecting to database", || {
            let db = SqliteConnection::establish(&path)
                .context("Failed to connect to database")?;
            migrations::run(&db)
                .context("Failed to run migrations")?;
            Ok(db)
        }, false)?;

        Ok(Database {
            name,
            db,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn db(&self) -> &SqliteConnection {
        &self.db
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_generic(&self, object: &Object) -> Result<bool> {
        match object {
            Object::Subdomain(object) => self.insert_subdomain_struct(&NewSubdomain {
                domain_id: object.domain_id,
                value: &object.value,
            }),
        }
    }

    pub fn insert_domain(&self, domain: &str) -> Result<()> {
        let new_domain = NewDomain {
            value: domain,
        };

        diesel::insert_into(domains::table)
            .values(&new_domain)
            .execute(&self.db)?;

        Ok(())
    }

    pub fn list_domains(&self) -> Result<Vec<Domain>> {
        use schema::domains::dsl::*;

        let results = domains.load::<Domain>(&self.db)?;

        Ok(results)
    }

    pub fn domain(&self, domain: &str) -> Result<i32> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(&domain))
            .select(id)
            .first::<i32>(&self.db)?;

        Ok(domain_id)
    }

    pub fn domain_optional(&self, domain: &str) -> Result<Option<i32>> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(&domain))
            .select(id)
            .first::<i32>(&self.db)
            .optional()?;

        Ok(domain_id)
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain(&self, subdomain: &str, domain: &str) -> Result<bool> {
        let domain_id = match self.domain_optional(domain)? {
            Some(domain_id) => domain_id,
            None => {
                self.insert_domain(domain)?;
                self.domain(domain)?
            },
        };

        let new_subdomain = NewSubdomain {
            domain_id,
            value: &subdomain,
        };

        self.insert_subdomain_struct(&new_subdomain)
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain_struct(&self, subdomain: &NewSubdomain) -> Result<bool> {
        // upsert is not supported by diesel

        if let Some(_subdomain_id) = self.select_subdomain_optional(subdomain.value)? {
            // TODO: right now we don't have any fields to update
            Ok(false)
        } else {
            diesel::insert_into(subdomains::table)
                .values(subdomain)
                .execute(&self.db)?;
            Ok(true)
        }
    }

    pub fn list_subdomains(&self) -> Result<Vec<Subdomain>> {
        use schema::subdomains::dsl::*;

        let results = subdomains.load::<Subdomain>(&self.db)?;

        Ok(results)
    }

    pub fn select_subdomain_optional(&self, subdomain: &str) -> Result<Option<i32>> {
        use schema::subdomains::dsl::*;

        let subdomain_id = subdomains.filter(value.eq(&subdomain))
            .select(id)
            .first::<i32>(&self.db)
            .optional()?;

        Ok(subdomain_id)
    }
}
