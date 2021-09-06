use crate::errors::*;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::result;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Glob {
    patterns: Vec<glob::Pattern>,
    src: String,
    inverse: bool,
}

impl Glob {
    pub fn matches(&self, topic: &str) -> Option<bool> {
        let mut filter = self.patterns.iter();
        let mut topic = topic.split(':');

        loop {
            match (filter.next(), topic.next()) {
                (Some(filter), Some(topic)) => if !filter.matches(topic) {
                    return None;
                },
                (None, None) => return Some(!self.inverse),
                (_, _) => return None,
            }
        }
    }
}

impl FromStr for Glob {
    type Err = Error;

    fn from_str(s: &str) -> Result<Glob> {
        let (s, inverse) = if let Some(stripped) = s.strip_prefix('!') {
            (stripped, true)
        } else {
            (s, false)
        };

        let patterns = s.split(':')
            .map(|s| glob::Pattern::new(s).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        Ok(Glob {
            patterns,
            src: s.to_string(),
            inverse,
        })
    }
}

impl Serialize for Glob {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.src)
    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn match_topic_str(filter: &str, value: &str) -> bool {
        let filter: Glob = filter.parse().unwrap();
        filter.matches(value).unwrap_or(false)
    }

    #[test]
    fn test_match_topic_exact() {
        assert!(match_topic_str("topic:hello-world", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_starts_with() {
        assert!(match_topic_str("topic:*", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_ends_with() {
        assert!(match_topic_str("*:hello-world", "topic:hello-world"));
    }

    #[test]
    fn test_match_topic_one_wildcard_one_section() {
        assert!(match_topic_str("a:*:z", "a:b:z"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections() {
        assert!(!match_topic_str("a:*:z", "a:b:c:z"));
    }

    #[test]
    fn test_match_topic_two_wildcards_two_sections() {
        assert!(match_topic_str("a:*:*:z", "a:b:c:z"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections_start() {
        assert!(!match_topic_str("a:*", "a:b:c"));
    }

    #[test]
    fn test_match_topic_one_wildcard_not_two_sections_end() {
        assert!(!match_topic_str("*:z", "b:c:z"));
    }

    #[test]
    fn test_match_topic_many_wildcards() {
        assert!(match_topic_str("a:*:*:d:e:*:g:*:z", "a:b:c:d:e:f:g:h:z"));
    }

    #[test]
    fn test_match_topic_empty_filter() {
        assert!(!match_topic_str("", "abc"));
    }

    #[test]
    fn test_inverse_match() {
        assert!(!match_topic_str("!topic:*", "topic:hello-world"));
    }

    #[test]
    fn test_no_inverse_match() {
        // Just because an inverse rule doesn't match this doesn't imply a match
        assert!(!match_topic_str("!foo:*", "topic:hello-world"));
    }
}
