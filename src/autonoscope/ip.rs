use crate::errors::*;
use crate::autonoscope::{Autonoscope, ToRule, AutoRule, RulePrecision};
use crate::models::*;
use std::convert::TryFrom;
use std::net;
use ipnetwork::IpNetwork;


#[derive(Debug, PartialEq)]
pub struct IpRule {
    network: IpNetwork,
}

impl ToString for IpRule {
    fn to_string(&self) -> String {
        self.network.to_string()
    }
}

impl TryFrom<&str> for IpRule {
    type Error = Error;

    fn try_from(x: &str) -> Result<IpRule> {
        let network = x.parse::<IpNetwork>()?;
        Ok(IpRule {
            network,
        })
    }
}

impl TryFrom<Autonoscope> for IpRule {
    type Error = Error;

    fn try_from(x: Autonoscope) -> Result<IpRule> {
        IpRule::try_from(x.value.as_str())
    }
}

impl AutoRule<IpAddr> for IpRule {
    fn matches(&self, ipaddr: &IpAddr) -> Result<bool> {
        self.matches(ipaddr.value.as_str())
    }
}

impl AutoRule<NewIpAddr> for IpRule {
    fn matches(&self, ipaddr: &NewIpAddr) -> Result<bool> {
        self.matches(ipaddr.value.as_str())
    }
}

impl AutoRule<Port> for IpRule {
    fn matches(&self, port: &Port) -> Result<bool> {
        let addr = port.value.parse::<net::SocketAddr>()?;
        self.matches(&addr.ip())
    }
}

impl AutoRule<NewPort> for IpRule {
    fn matches(&self, port: &NewPort) -> Result<bool> {
        let addr = port.value.parse::<net::SocketAddr>()?;
        self.matches(&addr.ip())
    }
}

fn match_netblock_str(network: &IpNetwork, netblock: &str) -> Result<bool> {
    let range = netblock.parse::<ipnetwork::IpNetwork>()?;

    if network.prefix() <= range.prefix() {
        Ok(network.contains(range.ip()))
    } else {
        Ok(false)
    }
}

impl AutoRule<Netblock> for IpRule {
    fn matches(&self, netblock: &Netblock) -> Result<bool> {
        match_netblock_str(&self.network, &netblock.value)
    }
}

impl AutoRule<NewNetblock> for IpRule {
    fn matches(&self, netblock: &NewNetblock) -> Result<bool> {
        match_netblock_str(&self.network, &netblock.value)
    }
}

impl AutoRule<str> for IpRule {
    fn matches(&self, ipaddr: &str) -> Result<bool> {
        let ipaddr = ipaddr.parse::<net::IpAddr>()?;
        self.matches(&ipaddr)
    }
}

impl AutoRule<net::IpAddr> for IpRule {
    fn matches(&self, ipaddr: &net::IpAddr) -> Result<bool> {
        Ok(self.network.contains(*ipaddr))
    }
}

impl RulePrecision for IpRule {
    fn precision(&self) -> usize {
        self.network.prefix() as usize
    }
}

impl ToRule for IpRule {
    fn to_rule(&self) -> (&'static str, String) {
        ("ip", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_ip_rule_ipv4_root() {
        let rule = IpRule::try_from("0.0.0.0/0").unwrap();
        assert!(rule.matches("127.0.0.1").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_ip_rule_ipv6_root() {
        let rule = IpRule::try_from("::/0").unwrap();
        assert!(rule.matches("::1").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_ip_rule_ipv4_match() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(rule.matches("192.0.2.1").unwrap());
        assert_eq!(rule.precision(), 24);
    }

    #[test]
    fn test_ip_rule_ipv4_mismatch() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(!rule.matches("127.0.0.1").unwrap());
        assert_eq!(rule.precision(), 24);
    }

    #[test]
    fn test_ip_rule_ipv6_match() {
        let rule = IpRule::try_from("2001:db8::/32").unwrap();
        assert!(rule.matches("2001:DB8::1").unwrap());
        assert_eq!(rule.precision(), 32);
    }

    #[test]
    fn test_ip_rule_ipv6_mismatch() {
        let rule = IpRule::try_from("2001:db8::/32").unwrap();
        assert!(!rule.matches("::1").unwrap());
        assert_eq!(rule.precision(), 32);
    }

    #[test]
    fn test_ip_rule_ipv6_on_ipv4_mismatch() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(!rule.matches("2001:DB8::1").unwrap());
        assert_eq!(rule.precision(), 24);
    }

    #[test]
    fn test_ip_rule_ipv4_on_ipv6_mismatch() {
        let rule = IpRule::try_from("2001:db8::/32").unwrap();
        assert!(!rule.matches("192.0.2.1").unwrap());
        assert_eq!(rule.precision(), 32);
    }

    #[test]
    fn test_ip_rule_netblock_inner() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(rule.matches(&NewNetblock {
            family: String::from("4"),
            value: String::from("192.0.2.128/25"),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }).unwrap());
    }

    #[test]
    fn test_ip_rule_netblock_equal() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(rule.matches(&NewNetblock {
            family: String::from("4"),
            value: String::from("192.0.2.0/24"),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }).unwrap());
    }

    #[test]
    fn test_ip_rule_netblock_outer1() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(!rule.matches(&NewNetblock {
            family: String::from("4"),
            value: String::from("192.0.2.0/23"),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }).unwrap());
    }

    #[test]
    fn test_ip_rule_netblock_outer2() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(!rule.matches(&NewNetblock {
            family: String::from("4"),
            value: String::from("192.0.0.0/22"),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }).unwrap());
    }

    #[test]
    fn test_ip_rule_netblock_no_overlap() {
        let rule = IpRule::try_from("192.0.2.0/24").unwrap();
        assert!(!rule.matches(&NewNetblock {
            family: String::from("4"),
            value: String::from("192.0.3.0/24"),
            asn: None,
            as_org: None,
            description: None,
            unscoped: false,
        }).unwrap());
    }
}
