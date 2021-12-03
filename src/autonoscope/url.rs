use crate::errors::*;
use crate::autonoscope::{Autonoscope, ToRule, AutoRule, RulePrecision};
use crate::models::*;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct UrlRule {
    url: String,
    origin: url::Origin,
    segments: Vec<String>,
}

impl ToString for UrlRule {
    fn to_string(&self) -> String {
        self.url.clone()
    }
}

impl TryFrom<Autonoscope> for UrlRule {
    type Error = Error;

    fn try_from(x: Autonoscope) -> Result<UrlRule> {
        UrlRule::try_from(x.value.as_str())
    }
}

impl TryFrom<&str> for UrlRule {
    type Error = Error;

    fn try_from(x: &str) -> Result<UrlRule> {
        let url = x.parse::<url::Url>()?;

        let origin = url.origin();

        let segments = url.path_segments()
            .ok_or_else(|| format_err!("url can't have a base"))?
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect();

        Ok(UrlRule {
            url: x.to_string(),
            origin,
            segments,
        })
    }
}

// TODO: there is no way to write a rule that matches all urls
impl AutoRule<Url> for UrlRule {
    fn matches(&self, url: &Url) -> Result<bool> {
        self.matches(url.value.as_str())
    }
}

impl AutoRule<NewUrl> for UrlRule {
    fn matches(&self, url: &NewUrl) -> Result<bool> {
        self.matches(url.value.as_str())
    }
}

impl AutoRule<str> for UrlRule {
    fn matches(&self, url: &str) -> Result<bool> {
        let url = url.parse::<url::Url>()?;

        if url.origin() != self.origin {
            return Ok(false);
        }

        let segments = url.path_segments()
            .ok_or_else(|| format_err!("url can't have a base"))?
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();

        if self.segments.len() > segments.len() {
            return Ok(false);
        }

        for (rule, path) in self.segments.iter().zip(segments.iter()) {
            if rule != path {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl RulePrecision for UrlRule {
    fn precision(&self) -> usize {
        self.segments.len()
    }
}

impl ToRule for UrlRule {
    fn to_rule(&self) -> (&'static str, String) {
        ("url", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_url_rule_explicit_origin() {
        let rule = UrlRule::try_from("https://example.com").unwrap();
        assert!(rule.matches("https://example.com:443/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_scheme_mismatch() {
        let rule = UrlRule::try_from("https://example.com").unwrap();
        assert!(!rule.matches("http://example.com:443/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_port_mismatch() {
        let rule = UrlRule::try_from("https://example.com").unwrap();
        assert!(!rule.matches("https://example.com:80/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_subdomain_mismatch1() {
        let rule = UrlRule::try_from("https://example.com").unwrap();
        assert!(!rule.matches("https://www.example.com/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_subdomain_mismatch2() {
        let rule = UrlRule::try_from("https://www.example.com").unwrap();
        assert!(!rule.matches("https://example.com/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_ftp() {
        let rule = UrlRule::try_from("ftp://www.example.com").unwrap();
        assert!(!rule.matches("https://example.com/").unwrap());
        assert_eq!(rule.precision(), 0);
    }

    #[test]
    fn test_url_rule_outside_of_path() {
        let rule = UrlRule::try_from("https://www.example.com/asset").unwrap();
        assert!(!rule.matches("https://example.com/").unwrap());
        assert_eq!(rule.precision(), 1);
    }

    #[test]
    fn test_url_rule_path_match_implicit_slash() {
        let rule = UrlRule::try_from("https://www.example.com/asset").unwrap();
        assert!(rule.matches("https://www.example.com/asset/").unwrap());
        assert_eq!(rule.precision(), 1);
    }

    #[test]
    fn test_url_rule_path_match_explicit_slash() {
        let rule = UrlRule::try_from("https://www.example.com/asset/").unwrap();
        assert!(rule.matches("https://www.example.com/asset").unwrap());
        assert_eq!(rule.precision(), 1);
    }

    #[test]
    fn test_url_rule_in_folder_implicit_slash() {
        let rule = UrlRule::try_from("https://www.example.com/asset").unwrap();
        assert!(rule.matches("https://www.example.com/asset/style.css").unwrap());
        assert_eq!(rule.precision(), 1);
    }

    #[test]
    fn test_url_rule_in_folder_explicit_slash() {
        let rule = UrlRule::try_from("https://www.example.com/asset/").unwrap();
        assert!(rule.matches("https://www.example.com/asset/style.css").unwrap());
        assert_eq!(rule.precision(), 1);
    }
}
