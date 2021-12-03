use crate::errors::*;
use crate::autonoscope::{Autonoscope, ToRule, AutoRule, RulePrecision};
use crate::models::*;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct DomainRule {
    value: String,
    fragments: Vec<String>,
}

impl ToString for DomainRule {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}

impl TryFrom<&str> for DomainRule {
    type Error = Error;

    fn try_from(rule: &str) -> Result<DomainRule> {
        let mut fragments = rule.split('.')
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<_>>();
        fragments.reverse();

        Ok(DomainRule {
            value: rule.to_string(),
            fragments,
        })
    }
}

impl TryFrom<Autonoscope> for DomainRule {
    type Error = Error;

    #[inline]
    fn try_from(rule: Autonoscope) -> Result<DomainRule> {
        DomainRule::try_from(rule.value.as_str())
    }
}

impl AutoRule<Domain> for DomainRule {
    #[inline]
    fn matches(&self, domain: &Domain) -> Result<bool> {
        self.matches(domain.value.as_str())
    }
}

impl AutoRule<NewDomain> for DomainRule {
    #[inline]
    fn matches(&self, domain: &NewDomain) -> Result<bool> {
        self.matches(domain.value.as_str())
    }
}

impl AutoRule<Subdomain> for DomainRule {
    #[inline]
    fn matches(&self, domain: &Subdomain) -> Result<bool> {
        self.matches(domain.value.as_str())
    }
}

impl AutoRule<NewSubdomain> for DomainRule {
    #[inline]
    fn matches(&self, domain: &NewSubdomain) -> Result<bool> {
        self.matches(domain.value.as_str())
    }
}

impl AutoRule<Url> for DomainRule {
    #[inline]
    fn matches(&self, url: &Url) -> Result<bool> {
        let url = url.value.parse::<url::Url>()?;
        if let Some(domain) = url.domain() {
            self.matches(domain)
        } else {
            Ok(false)
        }
    }
}

impl AutoRule<NewUrl> for DomainRule {
    #[inline]
    fn matches(&self, url: &NewUrl) -> Result<bool> {
        let url = url.value.parse::<url::Url>()?;
        if let Some(domain) = url.domain() {
            self.matches(domain)
        } else {
            Ok(false)
        }
    }
}

impl AutoRule<str> for DomainRule {
    fn matches(&self, domain: &str) -> Result<bool> {
        let frags = domain.split('.')
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();

        if self.fragments.len() > frags.len() {
            return Ok(false);
        }

        for (rule, domain) in self.fragments.iter().zip(frags.iter().rev()) {
            if rule != domain {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl RulePrecision for DomainRule {
    #[inline]
    fn precision(&self) -> usize {
        self.fragments.len()
    }
}

impl ToRule for DomainRule {
    fn to_rule(&self) -> (&'static str, String) {
        ("domain", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_domain_rule_root() {
        let rule = DomainRule::try_from(".").unwrap();
        assert!(rule.matches("example.com").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_domain_rule_com() {
        let rule = DomainRule::try_from("com").unwrap();
        assert!(rule.matches("example.com").unwrap());
        assert_eq!(rule.precision(), 1);
    }

    #[test]
    fn test_domain_rule_equals() {
        let rule = DomainRule::try_from("example.com").unwrap();
        assert!(rule.matches("example.com").unwrap());
        assert_eq!(rule.precision(), 2);
    }

    #[test]
    fn test_domain_rule_mismatch() {
        let rule = DomainRule::try_from("foo.example.com").unwrap();
        assert!(!rule.matches("example.com").unwrap());
        assert_eq!(rule.precision(), 3);
    }
}
