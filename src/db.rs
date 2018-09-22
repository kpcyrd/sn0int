use errors::*;

use diesel;
use diesel::expression::SqlLiteral;
use diesel::expression::sql_literal::sql;
use diesel::sql_types::Bool;
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
    pub fn insert_generic(&self, object: &Object) -> Result<(bool, i32)> {
        match object {
            Object::Subdomain(object) => self.insert_subdomain_struct(&NewSubdomain {
                domain_id: object.domain_id,
                value: &object.value,
            }),
            Object::IpAddr(object) => self.insert_ipaddr_struct(&NewIpAddr {
                family: &object.family,
                value: &object.value,
            }),
            Object::SubdomainIpAddr(object) => self.insert_subdomain_ipaddr_struct(&NewSubdomainIpAddr {
                subdomain_id: object.subdomain_id,
                ip_addr_id: object.ip_addr_id,
            }),
            Object::Url(object) => self.insert_url_struct(&NewUrl {
                subdomain_id: object.subdomain_id,
                value: &object.value,
                status: object.status,
                body: object.body.as_ref().map(|x| x.as_ref()),
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

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain(&self, subdomain: &str, domain: &str) -> Result<(bool, i32)> {
        let domain_id = match Domain::id_opt(self, domain)? {
            Some(domain_id) => domain_id,
            None => {
                self.insert_domain(domain)?;
                Domain::id(self, domain)?
            },
        };

        let new_subdomain = NewSubdomain {
            domain_id,
            value: &subdomain,
        };

        self.insert_subdomain_struct(&new_subdomain)
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain_struct(&self, subdomain: &NewSubdomain) -> Result<(bool, i32)> {
        // upsert is not supported by diesel

        if let Some(subdomain_id) = Subdomain::id_opt(self, subdomain.value)? {
            // TODO: right now we don't have any fields to update
            Ok((false, subdomain_id))
        } else {
            diesel::insert_into(subdomains::table)
                .values(subdomain)
                .execute(&self.db)?;
            let id = Subdomain::id(self, subdomain.value)?;
            Ok((true, id))
        }
    }

    pub fn insert_ipaddr(&self, family: &str, ipaddr: &str) -> Result<(bool, i32)> {
        // TODO: maybe check if valid
        let new_ipaddr = NewIpAddr {
            family: &family,
            value: &ipaddr,
        };

        self.insert_ipaddr_struct(&new_ipaddr)
    }

    pub fn insert_ipaddr_struct(&self, ipaddr: &NewIpAddr) -> Result<(bool, i32)> {
        // upsert is not supported by diesel

        if let Some(ipaddr_id) = IpAddr::id_opt(self, ipaddr.value)? {
            // TODO: right now we don't have any fields to update
            Ok((false, ipaddr_id))
        } else {
            diesel::insert_into(ipaddrs::table)
                .values(ipaddr)
                .execute(&self.db)?;
            let id = IpAddr::id(self, ipaddr.value)?;
            Ok((true, id))
        }
    }

    pub fn insert_subdomain_ipaddr(&self, subdomain_id: i32, ip_addr_id: i32) -> Result<(bool, i32)> {
        self.insert_subdomain_ipaddr_struct(&NewSubdomainIpAddr {
            subdomain_id,
            ip_addr_id,
        })
    }

    pub fn insert_subdomain_ipaddr_struct(&self, subdomain_ipaddr: &NewSubdomainIpAddr) -> Result<(bool, i32)> {
        if let Some(subdomain_ipaddr_id) = SubdomainIpAddr::id_opt(self, &(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id))? {
            Ok((false, subdomain_ipaddr_id))
        } else {
            diesel::insert_into(subdomain_ipaddrs::table)
                .values(subdomain_ipaddr)
                .execute(&self.db)?;
            let id = SubdomainIpAddr::id(self, &(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id))?;
            Ok((true, id))
        }
    }

    pub fn insert_url_struct(&self, url: &NewUrl) -> Result<(bool, i32)> {
        if let Some(url_id) = Url::id_opt(self, url.value)? {
            Ok((false, url_id))
        } else {
            diesel::insert_into(urls::table)
                .values(url)
                .execute(&self.db)?;
            let id = Url::id(self, url.value)?;
            Ok((true, id))
        }
    }

    //

    pub fn list<T: Model>(&self) -> Result<Vec<T>> {
        T::list(self)
    }

    pub fn filter<T: Model>(&self, filter: &Filter) -> Result<Vec<T>> {
        T::filter(self, filter)
    }
}

#[derive(Debug, PartialEq)]
pub struct Filter {
    query: String,
}

impl Filter {
    pub fn new<I: Into<String>>(query: I) -> Filter {
        Filter {
            query: query.into(),
        }
    }

    pub fn parse(mut args: &[String]) -> Result<Filter> {
        debug!("Parsing query: {:?}", args);

        if args.is_empty() {
            return Ok(Filter::new("1"));
        }

        if args[0].to_lowercase() == "where" {
            args = &args[1..];
        } else {
            bail!("Filter must begin with WHERE");
        }

        let mut query = String::new();

        let mut expect_value = false;

        for arg in args {
            if let Some(idx) = arg.find("=") {
                if idx != 0 {
                    let (key, value) = arg.split_at(idx);
                    query += &format!(" {} = {:?}", key, &value[1..]);
                    continue;
                }
            }

            if expect_value {
                query += &format!(" {:?}", arg);
                expect_value = false;
            } else {
                if ["=", "!=", "like"].contains(&arg.to_lowercase().as_str()) {
                    expect_value = true;
                }

                query += &format!(" {}", arg);
            }
        }
        debug!("Parsed query: {:?}", query);

        Ok(Filter::new(query))
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn sql(&self) -> SqlLiteral<Bool> {
        sql::<Bool>(&self.query)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_simple() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=1".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"1\""));
    }

    #[test]
    fn test_filter_str1() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=abc".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"abc\""));
    }

    #[test]
    fn test_filter_str2() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "asdf".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"asdf\""));
    }

    #[test]
    fn test_filter_and() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "foobar".to_string(),
                                     "and".to_string(),
                                     "id".to_string(),
                                     "=".to_string(),
                                     "1".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"foobar\" and id = \"1\""));
    }

    #[test]
    fn test_filter_like() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "like".to_string(),
                                     "%foobar".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value like \"%foobar\""));
    }
}
