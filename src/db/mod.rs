use crate::errors::*;
use serde::{Serialize, Deserialize};

use diesel::expression::SqlLiteral;
use diesel::expression::sql_literal::sql;
use diesel::sql_types::Bool;
use diesel::prelude::*;
use std::fmt::Write;
use strum_macros::{EnumString, IntoStaticStr};
use crate::autonoscope::{RuleSet, RuleType};
use crate::models::*;
use crate::schema::*;
use crate::migrations;
use crate::worker;
use crate::workspaces::Workspace;

pub mod ttl;


#[derive(Debug)]
pub enum DbChange {
    Insert,
    Update(Update),
    None,
}

impl DbChange {
    pub fn is_some(&self) -> bool {
        !matches!(self, DbChange::None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(EnumString, IntoStaticStr)]
#[strum(serialize_all = "kebab_case")]
pub enum Family {
    Domain,
    Subdomain,
    Ipaddr,
    SubdomainIpaddr,
    Url,
    Email,
    Phonenumber,
    Device,
    Network,
    NetworkDevice,
    Account,
    Breach,
    BreachEmail,
    Image,
    Port,
    Netblock,
    Cryptoaddr,
}

impl Family {
    #[inline(always)]
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

#[derive(EnumString, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum Table {
    Domains,
    Subdomains,
    Ipaddrs,
    SubdomainIpaddrs,
    Urls,
    Emails,
    Phonenumbers,
    Devices,
    Networks,
    NetworkDevices,
    Accounts,
    Breaches,
    BreachEmails,
    Images,
    Ports,
    Netblocks,
    Cryptoaddrs,
}

impl Table {
    #[inline(always)]
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}

pub struct Database {
    workspace: Workspace,
    db: SqliteConnection,
    autonoscope: RuleSet,
}

pub type DatabaseSock = diesel::SqliteConnection;

impl Database {
    pub fn establish(workspace: Workspace) -> Result<Database> {
        let db = worker::spawn_fn("Connecting to database", || {
            Database::establish_quiet(workspace)
        }, false)?;

        Ok(db)
    }

    pub fn establish_quiet(workspace: Workspace) -> Result<Database> {
        let path = workspace.db_path()?;
        let path = path.into_os_string().into_string()
            .map_err(|_| format_err!("Failed to convert db path to utf-8"))?;

        let db = SqliteConnection::establish(&path)
            .context("Failed to connect to database")?;

        db.execute("PRAGMA busy_timeout = 10000")
            .context("Failed to set busy_timeout")?;
        db.execute("PRAGMA foreign_keys = ON")
            .context("Failed to enforce foreign keys")?;
        db.execute("PRAGMA journal_mode = WAL")
            .context("Failed to enable write ahead log")?;
        db.execute("PRAGMA synchronous = NORMAL")
            .context("Failed to enforce foreign keys")?;

        migrations::run(&db)
            .context("Failed to run migrations")?;

        let autonoscope = RuleSet::load(&db)?;

        Ok(Database {
            workspace,
            db,
            autonoscope,
        })
    }

    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.workspace
    }

    #[inline(always)]
    pub fn db(&self) -> &SqliteConnection {
        &self.db
    }

    #[inline(always)]
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    #[inline(always)]
    pub fn autonoscope_add_rule(&mut self, object: &RuleType, value: &str, scoped: bool) -> Result<()> {
        self.autonoscope.add_rule(&self.db, object, value, scoped)
    }

    #[inline(always)]
    pub fn autonoscope_delete_rule(&mut self, object: &RuleType, value: &str) -> Result<()> {
        self.autonoscope.delete_rule(&self.db, object, value)
    }

    #[inline(always)]
    pub fn autonoscope_rules(&self) -> Vec<(&'static str, String, bool)> {
        self.autonoscope.rules()
    }

    #[inline(always)]
    pub fn autonoscope(&self) -> &RuleSet {
        &self.autonoscope
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_generic(&self, object: Insert) -> Result<Option<(DbChange, i32)>> {
        let scoped = self.autonoscope.matches(&object)?;
        match object {
            Insert::Domain(object) => self.insert_struct(object, scoped),
            Insert::Subdomain(object) => self.insert_struct(object, scoped),
            Insert::IpAddr(object) => self.insert_struct(object, scoped),
            Insert::SubdomainIpAddr(object) => self.insert_subdomain_ipaddr_struct(&object),
            Insert::Url(object) => self.insert_struct(object, scoped),
            Insert::Email(object) => self.insert_struct(object, scoped),
            Insert::PhoneNumber(object) => self.insert_struct(object, scoped),
            Insert::Device(object) => self.insert_struct(object, scoped),
            Insert::Network(object) => self.insert_struct(object, scoped),
            Insert::NetworkDevice(object) => self.insert_network_device_struct(&object),
            Insert::Account(object) => self.insert_struct(object, scoped),
            Insert::Breach(object) => self.insert_struct(object, scoped),
            Insert::BreachEmail(object) => self.insert_breach_email_struct(object),
            Insert::Image(object) => self.insert_struct(object, scoped),
            Insert::Port(object) => self.insert_struct(object, scoped),
            Insert::Netblock(object) => self.insert_struct(object, scoped),
            Insert::CryptoAddr(object) => self.insert_struct(object, scoped),
        }
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_struct<T: InsertableStruct<M>, M: Model + Scopable>(&self, mut obj: T, scoped: bool) -> Result<Option<(DbChange, i32)>> {
        if let Some(existing) = M::get_opt(self, obj.value())? {
            // entity is out of scope
            if !existing.scoped() {
                return Ok(None);
            }

            let update = obj.upsert(&existing);
            if update.is_dirty() {
                update.apply(self)?;
                Ok(Some((DbChange::Update(update.generic()), existing.id())))
            } else {
                Ok(Some((DbChange::None, existing.id())))
            }
        } else {
            obj.set_scoped(scoped);
            obj.insert(self)?;
            let id = M::get_id(self, obj.value())?;
            Ok(Some((DbChange::Insert, id)))
        }
    }

    pub fn insert_activity(&self, obj: NewActivity) -> Result<bool> {
        if let Some(uniq) = &obj.uniq {
            if Activity::uniq(self, uniq)?.is_some() {
                // unique tag set and event already logged
                return Ok(false);
            }
        }
        obj.insert(self)?;
        Ok(true)
    }

    pub fn insert_subdomain_ipaddr_struct(&self, subdomain_ipaddr: &NewSubdomainIpAddr) -> Result<Option<(DbChange, i32)>> {
        if let Some(subdomain_ipaddr_id) = SubdomainIpAddr::get_id_opt(self, &(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id))? {
            Ok(Some((DbChange::None, subdomain_ipaddr_id)))
        } else {
            diesel::insert_into(subdomain_ipaddrs::table)
                .values(subdomain_ipaddr)
                .execute(&self.db)?;
            let id = SubdomainIpAddr::get_id(self, &(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id))?;
            Ok(Some((DbChange::Insert, id)))
        }
    }

    pub fn insert_network_device_struct(&self, network_device: &NewNetworkDevice) -> Result<Option<(DbChange, i32)>> {
        if let Some(network_device_id) = NetworkDevice::get_id_opt(self, &(network_device.network_id, network_device.device_id))? {
            Ok(Some((DbChange::None, network_device_id)))
        } else {
            diesel::insert_into(network_devices::table)
                .values(network_device)
                .execute(&self.db)?;
            let id = NetworkDevice::get_id(self, &(network_device.network_id, network_device.device_id))?;
            Ok(Some((DbChange::Insert, id)))
        }
    }

    pub fn insert_breach_email_struct(&self, obj: NewBreachEmail) -> Result<Option<(DbChange, i32)>> {
        let value = &(obj.breach_id, obj.email_id, obj.password.clone());

        if let Some(existing) = BreachEmail::get_opt(self, value)? {
            let id = <BreachEmail as Model>::id(&existing);

            let update = obj.upsert(&existing);
            if update.is_dirty() {
                update.apply(self)?;
                Ok(Some((DbChange::Update(update.generic()), id)))
            } else {
                Ok(Some((DbChange::None, id)))
            }
        } else {
            diesel::insert_into(breach_emails::table)
                .values(obj)
                .execute(&self.db)?;
            let id = BreachEmail::get_id(self, value)?;
            Ok(Some((DbChange::Insert, id)))
        }
    }

    //

    pub fn update_generic(&self, update: &Update) -> Result<i32> {
        match update {
            Update::Subdomain(update) => self.update_subdomain(update),
            Update::IpAddr(update) => self.update_ipaddr(update),
            Update::Url(update) => self.update_url(update),
            Update::Email(update) => self.update_email(update),
            Update::PhoneNumber(update) => self.update_phonenumber(update),
            Update::Device(update) => self.update_device(update),
            Update::Network(update) => self.update_network(update),
            Update::NetworkDevice(update) => self.update_network_device(update),
            Update::Account(update) => self.update_account(update),
            Update::BreachEmail(update) => self.update_breach_email(update),
            Update::Image(update) => self.update_image(update),
            Update::Port(update) => self.update_port(update),
            Update::Netblock(update) => self.update_netblock(update),
            Update::CryptoAddr(update) => self.update_cryptoaddr(update),
        }
    }

    pub fn update_subdomain(&self, subdomain_update: &SubdomainUpdate) -> Result<i32> {
        use crate::schema::subdomains::columns::*;
        diesel::update(subdomains::table.filter(id.eq(subdomain_update.id)))
            .set(subdomain_update)
            .execute(&self.db)?;
        Ok(subdomain_update.id)
    }

    pub fn update_ipaddr(&self, ipaddr_update: &IpAddrUpdate) -> Result<i32> {
        use crate::schema::ipaddrs::columns::*;
        diesel::update(ipaddrs::table.filter(id.eq(ipaddr_update.id)))
            .set(ipaddr_update)
            .execute(&self.db)?;
        Ok(ipaddr_update.id)
    }

    pub fn update_url(&self, url_update: &UrlChangeset) -> Result<i32> {
        use crate::schema::urls::columns::*;
        diesel::update(urls::table.filter(id.eq(url_update.id)))
            .set(url_update)
            .execute(&self.db)?;
        Ok(url_update.id)
    }

    pub fn update_email(&self, email_update: &EmailUpdate) -> Result<i32> {
        use crate::schema::emails::columns::*;
        diesel::update(emails::table.filter(id.eq(email_update.id)))
            .set(email_update)
            .execute(&self.db)?;
        Ok(email_update.id)
    }

    pub fn update_phonenumber(&self, phonenumber_update: &PhoneNumberUpdate) -> Result<i32> {
        use crate::schema::phonenumbers::columns::*;
        diesel::update(phonenumbers::table.filter(id.eq(phonenumber_update.id)))
            .set(phonenumber_update)
            .execute(&self.db)?;
        Ok(phonenumber_update.id)
    }

    pub fn update_device(&self, device_update: &DeviceUpdate) -> Result<i32> {
        use crate::schema::devices::columns::*;
        diesel::update(devices::table.filter(id.eq(device_update.id)))
            .set(device_update)
            .execute(&self.db)?;
        Ok(device_update.id)
    }

    pub fn update_network(&self, network_update: &NetworkUpdate) -> Result<i32> {
        use crate::schema::networks::columns::*;
        diesel::update(networks::table.filter(id.eq(network_update.id)))
            .set(network_update)
            .execute(&self.db)?;
        Ok(network_update.id)
    }

    pub fn update_network_device(&self, network_device_update: &NetworkDeviceUpdate) -> Result<i32> {
        use crate::schema::network_devices::columns::*;
        diesel::update(network_devices::table.filter(id.eq(network_device_update.id)))
            .set(network_device_update)
            .execute(&self.db)?;
        Ok(network_device_update.id)
    }

    pub fn update_account(&self, account_update: &AccountUpdate) -> Result<i32> {
        use crate::schema::accounts::columns::*;
        diesel::update(accounts::table.filter(id.eq(account_update.id)))
            .set(account_update)
            .execute(&self.db)?;
        Ok(account_update.id)
    }

    pub fn update_breach_email(&self, breach_email_update: &BreachEmailUpdate) -> Result<i32> {
        use crate::schema::breach_emails::columns::*;
        diesel::update(breach_emails::table.filter(id.eq(breach_email_update.id)))
            .set(breach_email_update)
            .execute(&self.db)?;
        Ok(breach_email_update.id)
    }

    pub fn update_image(&self, image_update: &ImageUpdate) -> Result<i32> {
        use crate::schema::images::columns::*;
        diesel::update(images::table.filter(id.eq(image_update.id)))
            .set(image_update)
            .execute(&self.db)?;
        Ok(image_update.id)
    }

    pub fn update_port(&self, port_update: &PortUpdate) -> Result<i32> {
        use crate::schema::ports::columns::*;
        diesel::update(ports::table.filter(id.eq(port_update.id)))
            .set(port_update)
            .execute(&self.db)?;
        Ok(port_update.id)
    }

    pub fn update_netblock(&self, netblock_update: &NetblockUpdate) -> Result<i32> {
        use crate::schema::netblocks::columns::*;
        diesel::update(netblocks::table.filter(id.eq(netblock_update.id)))
            .set(netblock_update)
            .execute(&self.db)?;
        Ok(netblock_update.id)
    }

    pub fn update_cryptoaddr(&self, cryptoaddr_update: &CryptoAddrUpdate) -> Result<i32> {
        use crate::schema::cryptoaddrs::columns::*;
        diesel::update(cryptoaddrs::table.filter(id.eq(cryptoaddr_update.id)))
            .set(cryptoaddr_update)
            .execute(&self.db)?;
        Ok(cryptoaddr_update.id)
    }

    fn get_opt_typed<T: Model + Scopable>(&self, value: &T::ID) -> Result<Option<i32>> {
        match T::get_opt(self, value)? {
            Some(ref obj) if obj.scoped() => Ok(Some(obj.id())),
            _ => Ok(None),
        }
    }

    pub fn get_opt(&self, family: &Family, value: &str) -> Result<Option<i32>> {
        match family {
            Family::Domain => self.get_opt_typed::<Domain>(value),
            Family::Subdomain => self.get_opt_typed::<Subdomain>(value),
            Family::Ipaddr => self.get_opt_typed::<IpAddr>(value),
            Family::SubdomainIpaddr => bail!("Unsupported operation"),
            Family::Url => self.get_opt_typed::<Url>(value),
            Family::Email => self.get_opt_typed::<Email>(value),
            Family::Phonenumber => self.get_opt_typed::<PhoneNumber>(value),
            Family::Device => self.get_opt_typed::<Device>(value),
            Family::Network => self.get_opt_typed::<Network>(value),
            Family::NetworkDevice => bail!("Unsupported operation"),
            Family::Account => self.get_opt_typed::<Account>(value),
            Family::Breach => self.get_opt_typed::<Breach>(value),
            Family::BreachEmail => bail!("Unsupported operation"),
            Family::Image => self.get_opt_typed::<Image>(value),
            Family::Port => self.get_opt_typed::<Port>(value),
            Family::Netblock => self.get_opt_typed::<Netblock>(value),
            Family::Cryptoaddr => self.get_opt_typed::<CryptoAddr>(value),
        }
    }

    //

    pub fn list<T: Model>(&self) -> Result<Vec<T>> {
        T::list(self)
    }

    pub fn filter<T: Model>(&self, filter: &Filter) -> Result<Vec<T>> {
        T::filter(self, filter)
    }

    pub fn filter_with_param<T: Model>(&self, filter: &Filter, param: Option<&String>) -> Result<Vec<T>> {
        match param {
            Some(param) => T::filter_with_param(self, filter, param),
            _ => T::filter(self, filter),
        }
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

    #[inline]
    pub fn any() -> Filter {
        Filter::new("1")
    }

    fn escape(value: &str) -> String {
        let mut out = String::from("'");
        for c in value.chars() {
            match c {
                '\'' => out.push_str("''"),
                c => out.push(c),
            }
        }
        out.push('\'');
        out
    }

    pub fn parse(mut args: &[String]) -> Result<Filter> {
        debug!("Parsing query: {:?}", args);

        if args.is_empty() {
            bail!("Filter condition is required");
        }

        if args[0].to_lowercase() == "where" {
            args = &args[1..];
        } else {
            bail!("Filter must begin with WHERE");
        }

        let mut query = String::new();

        let mut expect_value = false;

        for arg in args {
            if ["=", "!=", "<", ">", "<=", ">=", "like"].contains(&arg.to_lowercase().as_str()) {
                expect_value = true;
                write!(query, " {}", arg)?;
                continue;
            }

            if let Some(idx) = arg.find('=') {
                if idx != 0 {
                    let (key, value) = arg.split_at(idx);
                    write!(query, " {} = {}", key, Self::escape(&value[1..]))?;
                    continue;
                }
            }

            if expect_value {
                query.push(' ');
                query.push_str(&Self::escape(arg));
                expect_value = false;
            } else {
                write!(query, " {}", arg)?;
            }
        }
        debug!("Parsed query: {:?}", query);

        Ok(Filter::new(query))
    }

    pub fn parse_optional(args: &[String]) -> Result<Filter> {
        debug!("Parsing optional query: {:?}", args);

        if args.is_empty() {
            debug!("Using filter with no condition");
            return Ok(Filter::any());
        }

        Self::parse(args)
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn and_scoped(&self) -> Filter {
        let query = format!("({}) AND unscoped=0", self.query);
        Filter::new(query)
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
        assert_eq!(filter, Filter::new(" value = '1'"));
    }

    #[test]
    fn test_filter_str1() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=abc".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = 'abc'"));
    }

    #[test]
    fn test_filter_str2() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "asdf".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = 'asdf'"));
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
        assert_eq!(filter, Filter::new(" value = 'foobar' and id = '1'"));
    }

    #[test]
    fn test_filter_like() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "like".to_string(),
                                     "%foobar".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value like '%foobar'"));
    }

    #[test]
    fn test_filter_backslash1() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=\\".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = '\\'"));
    }

    #[test]
    fn test_filter_backslash2() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "\\".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = '\\'"));
    }

    #[test]
    fn test_filter_quote1() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=a'b".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = 'a''b'"));
    }

    #[test]
    fn test_filter_quote2() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "a'b".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = 'a''b'"));
    }

    #[test]
    fn test_filter_greater() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     ">".to_string(),
                                     "123".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value > '123'"));
    }

    #[test]
    fn test_filter_smaller() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "<".to_string(),
                                     "123".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value < '123'"));
    }

    #[test]
    fn test_filter_greater_equal() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     ">=".to_string(),
                                     "123".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value >= '123'"));
    }

    #[test]
    fn test_filter_smaller_equal() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "<=".to_string(),
                                     "123".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value <= '123'"));
    }

    #[test]
    fn test_family_names() {
        assert_eq!(Family::Domain.as_str(),             "domain");
        assert_eq!(Family::Subdomain.as_str(),          "subdomain");
        assert_eq!(Family::Ipaddr.as_str(),             "ipaddr");
        assert_eq!(Family::SubdomainIpaddr.as_str(),    "subdomain-ipaddr");
        assert_eq!(Family::Url.as_str(),                "url");
        assert_eq!(Family::Email.as_str(),              "email");
        assert_eq!(Family::Phonenumber.as_str(),        "phonenumber");
        assert_eq!(Family::Device.as_str(),             "device");
        assert_eq!(Family::Network.as_str(),            "network");
        assert_eq!(Family::NetworkDevice.as_str(),      "network-device");
        assert_eq!(Family::Account.as_str(),            "account");
        assert_eq!(Family::Breach.as_str(),             "breach");
        assert_eq!(Family::BreachEmail.as_str(),        "breach-email");
        assert_eq!(Family::Image.as_str(),              "image");
        assert_eq!(Family::Port.as_str(),               "port");
        assert_eq!(Family::Netblock.as_str(),           "netblock");
    }

    #[test]
    fn test_table_names() {
        use super::Table;
        assert_eq!(Table::Domains.as_str(),             "domains");
        assert_eq!(Table::Subdomains.as_str(),          "subdomains");
        assert_eq!(Table::Ipaddrs.as_str(),             "ipaddrs");
        assert_eq!(Table::SubdomainIpaddrs.as_str(),    "subdomain_ipaddrs");
        assert_eq!(Table::Urls.as_str(),                "urls");
        assert_eq!(Table::Emails.as_str(),              "emails");
        assert_eq!(Table::Phonenumbers.as_str(),        "phonenumbers");
        assert_eq!(Table::Devices.as_str(),             "devices");
        assert_eq!(Table::Networks.as_str(),            "networks");
        assert_eq!(Table::NetworkDevices.as_str(),      "network_devices");
        assert_eq!(Table::Accounts.as_str(),            "accounts");
        assert_eq!(Table::Breaches.as_str(),            "breaches");
        assert_eq!(Table::BreachEmails.as_str(),        "breach_emails");
        assert_eq!(Table::Images.as_str(),              "images");
        assert_eq!(Table::Ports.as_str(),               "ports");
        assert_eq!(Table::Netblocks.as_str(),           "netblocks");
    }
}
