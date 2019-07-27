use crate::db::DatabaseSock;
use crate::errors::*;
use diesel;
use diesel::prelude::*;
use crate::schema::*;
use crate::models::*;
use std::cmp::Ordering;
use std::convert::{TryInto, TryFrom};
use std::str::FromStr;

mod domain;
pub use self::domain::DomainRule;
mod ip;
pub use self::ip::IpRule;
mod url;
pub use self::url::UrlRule;

#[derive(Identifiable, Queryable, PartialEq, Debug)]
#[table_name="autonoscope"]
pub struct Autonoscope {
    pub id: i32,
    pub object: String,
    pub value: String,
    pub scoped: bool,
}

#[derive(Insertable, PartialEq, Debug)]
#[table_name="autonoscope"]
pub struct NewAutonoscope {
    pub object: String,
    pub value: String,
    pub scoped: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct RuleSet {
    domains: Vec<Rule<DomainRule>>,
    ips: Vec<Rule<IpRule>>,
    urls: Vec<Rule<UrlRule>>,
}

#[inline(always)]
fn sort_precision_desc<T: RulePrecision>(a: &T, b: &T) -> Ordering {
    a.precision()
        .cmp(&b.precision())
        .reverse()
}

impl RuleSet {
    pub fn load(db: &DatabaseSock) -> Result<Self> {
        use crate::schema::autonoscope::dsl::*;
        let rules = autonoscope.load::<Autonoscope>(db)?;

        let mut set = RuleSet::default();
        for rule in rules {
            let is_scoped = rule.scoped;
            let rule_type = rule.object.parse::<RuleType>()?;
            match rule_type {
                RuleType::Domain => set.domains.push(Rule::new(rule.try_into()?, is_scoped)),
                RuleType::Ip => set.ips.push(Rule::new(rule.try_into()?, is_scoped)),
                RuleType::Url => set.urls.push(Rule::new(rule.try_into()?, is_scoped)),
            }
        }
        set.sort_rules();

        Ok(set)
    }

    fn sort_rules(&mut self) {
        self.domains.sort_by(sort_precision_desc);
        self.ips.sort_by(sort_precision_desc);
        self.urls.sort_by(sort_precision_desc);
    }

    pub fn add_rule(&mut self, db: &DatabaseSock, object: &RuleType, value: &str, scoped: bool) -> Result<()> {
        self.delete_rule(db, object, value)?;

        match object {
            RuleType::Domain => {
                let rule = DomainRule::try_from(value)?;
                let rule = Rule::new(rule, scoped);
                self.db_add(db, &rule)?;
                self.domains.push(rule);
            },
            RuleType::Ip => {
                let rule = IpRule::try_from(value)?;
                let rule = Rule::new(rule, scoped);
                self.db_add(db, &rule)?;
                self.ips.push(rule);
            },
            RuleType::Url => {
                let rule = UrlRule::try_from(value)?;
                let rule = Rule::new(rule, scoped);
                self.db_add(db, &rule)?;
                self.urls.push(rule);
            },
        }
        self.sort_rules();

        Ok(())
    }

    fn db_add<I: Into<NewAutonoscope>>(&mut self, db: &DatabaseSock, rule: I) -> Result<()> {
        use crate::schema::autonoscope::dsl::*;
        diesel::insert_into(autonoscope)
            .values(rule.into())
            .execute(db)?;
        Ok(())
    }

    pub fn delete_rule(&mut self, db: &DatabaseSock, obj: &RuleType, rule: &str) -> Result<()> {
        match obj {
            RuleType::Domain => {
                self.domains.retain(|x| x.to_string().as_str() != rule);
                self.db_delete(db, obj, &rule)?;
            },
            RuleType::Ip => {
                self.ips.retain(|x| x.to_string().as_str() != rule);
                self.db_delete(db, obj, &rule)?;
            },
            RuleType::Url => {
                self.urls.retain(|x| x.to_string().as_str() != rule);
                self.db_delete(db, obj, &rule)?;
            },
        }
        Ok(())
    }

    fn db_delete(&mut self, db: &DatabaseSock, obj: &RuleType, rule: &str) -> Result<()> {
        use crate::schema::autonoscope::dsl::*;
        diesel::delete(autonoscope
            .filter(object.eq(obj.as_str()))
            .filter(value.eq(rule)))
            .execute(db)?;
        Ok(())
    }

    pub fn rules(&self) -> Vec<(&'static str, String, bool)> {
        let mut rules = Vec::new();
        Self::push_rules_display(&mut rules, &self.domains);
        Self::push_rules_display(&mut rules, &self.ips);
        Self::push_rules_display(&mut rules, &self.urls);
        rules
    }

    #[inline]
    fn push_rules_display<T: IntoRule>(output: &mut Vec<(&'static str, String, bool)>, rules: &[Rule<T>]) {
        for rule in rules {
            let (object, value) = rule.into_rule();
            output.push((object, value, rule.scoped));
        }
    }

    pub fn matches(&self, object: &Insert) -> Result<bool> {
        let scoped = match object {
            Insert::Domain(domain) => Self::matches_any(&self.domains, domain)?,
            Insert::Subdomain(subdomain) => Self::matches_any(&self.domains, subdomain)?,
            Insert::IpAddr(ip_addr) => Self::matches_any(&self.ips, ip_addr)?,
            Insert::Url(url) => {
                if let Some(result) = Self::matches_any(&self.domains, url)? {
                    Some(result)
                } else if let Some(result) = Self::matches_any(&self.urls, url)? {
                    Some(result)
                } else {
                    None
                }
            },
            // Insert::Email(email) => unimplemented!(),
            // Insert::Account(account) => unimplemented!(),
            Insert::Port(port) => Self::matches_any(&self.ips, port)?,
            Insert::Netblock(netblock) => Self::matches_any(&self.ips, netblock)?,
            _ => None,
        };
        Ok(scoped.unwrap_or(true))
    }

    fn matches_any<T1, T2>(rules: &[Rule<T1>], object: &T2) -> Result<Option<bool>>
        where T1: AutoRule<T2>,
            T1: IntoRule,
    {
        for rule in rules {
            if rule.matches(object)? {
                return Ok(Some(rule.scoped));
            }
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub enum RuleType {
    Domain,
    Ip,
    Url,
}

impl RuleType {
    fn as_str(&self) -> &'static str {
        match self {
            RuleType::Domain => "domain",
            RuleType::Ip => "ip",
            RuleType::Url => "url",
        }
    }

    pub fn list_all() -> &'static [&'static str] {
        lazy_static! {
            static ref RULES: Vec<&'static str> = vec![
                RuleType::Domain.as_str(),
                RuleType::Ip.as_str(),
                RuleType::Url.as_str(),
            ];
        }

        RULES.as_ref()
    }
}

impl FromStr for RuleType {
    type Err = Error;

    fn from_str(s: &str) -> Result<RuleType> {
        match s {
            "domain" => Ok(RuleType::Domain),
            "ip" => Ok(RuleType::Ip),
            "url" => Ok(RuleType::Url),
            _ => bail!("unknown rule type"),
        }
    }
}

pub trait AutoRule<T: ?Sized> {
    fn matches(&self, value: &T) -> Result<bool>;
}

pub trait RulePrecision {
    fn precision(&self) -> usize;
}

#[derive(Debug, PartialEq)]
pub struct Rule<T: IntoRule> {
    rule: T,
    scoped: bool,
}

impl<T: IntoRule> Rule<T> {
    pub fn new(rule: T, scoped: bool) -> Rule<T> {
        Rule {
            rule,
            scoped,
        }
    }
}

// TODO: maybe drop this
use std::ops::Deref;
impl<T: IntoRule> Deref for Rule<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rule
    }
}

impl<T: IntoRule + RulePrecision> RulePrecision for Rule<T> {
    fn precision(&self) -> usize {
        self.rule.precision()
    }
}

pub trait IntoRule {
    fn into_rule(&self) -> (&'static str, String);
}

impl<T: IntoRule> Into<NewAutonoscope> for &Rule<T> {
    fn into(self) -> NewAutonoscope {
        let (object, value) = self.rule.into_rule();
        NewAutonoscope {
            object: object.to_string(),
            value,
            scoped: self.scoped,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_rule_sort_domains() {
        let mut set = RuleSet {
            domains: vec![
                Rule::new(DomainRule::try_from("com").unwrap(), true),
                Rule::new(DomainRule::try_from(".").unwrap(), true),
                Rule::new(DomainRule::try_from("example.com").unwrap(), true),
            ],
            ips: vec![],
            urls: vec![],
        };
        set.sort_rules();
        assert_eq!(set, RuleSet {
            domains: vec![
                Rule::new(DomainRule::try_from("example.com").unwrap(), true),
                Rule::new(DomainRule::try_from("com").unwrap(), true),
                Rule::new(DomainRule::try_from(".").unwrap(), true),
            ],
            ips: vec![],
            urls: vec![],
        });
    }

    #[test]
    fn test_rule_sort_ips() {
        let mut set = RuleSet {
            domains: vec![],
            ips: vec![
                Rule::new(IpRule::try_from("10.0.0.0/8").unwrap(), true),
                Rule::new(IpRule::try_from("0.0.0.0/0").unwrap(), true),
                Rule::new(IpRule::try_from("10.5.6.0/24").unwrap(), true),
            ],
            urls: vec![],
        };
        set.sort_rules();
        // TODO: add ipv6
        assert_eq!(set, RuleSet {
            domains: vec![],
            ips: vec![
                Rule::new(IpRule::try_from("10.5.6.0/24").unwrap(), true),
                Rule::new(IpRule::try_from("10.0.0.0/8").unwrap(), true),
                Rule::new(IpRule::try_from("0.0.0.0/0").unwrap(), true),
            ],
            urls: vec![],
        });
    }

    #[test]
    fn test_rule_sort_urls() {
        let mut set = RuleSet {
            domains: vec![],
            ips: vec![],
            urls: vec![
                Rule::new(UrlRule::try_from("http://example.com/foo/").unwrap(), true),
                Rule::new(UrlRule::try_from("https://example.com/").unwrap(), true),
                Rule::new(UrlRule::try_from("https://example.com/foo/bar/?asdf=1").unwrap(), true),
            ],
        };
        set.sort_rules();
        assert_eq!(set, RuleSet {
            domains: vec![],
            ips: vec![],
            urls: vec![
                Rule::new(UrlRule::try_from("https://example.com/foo/bar/?asdf=1").unwrap(), true),
                Rule::new(UrlRule::try_from("http://example.com/foo/").unwrap(), true),
                Rule::new(UrlRule::try_from("https://example.com/").unwrap(), true),
            ],
        });
    }
}
