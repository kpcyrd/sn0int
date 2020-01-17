#[derive(Debug, PartialEq)]
pub enum Token {
    Empty,
    Name(String),
    InString(StringState),
    Value(String),
}

#[derive(Debug, PartialEq)]
pub struct StringState {
    quote: char,
    escape: bool,
    buf: String,
}

impl StringState {
    pub fn new(quote: char) -> StringState {
        StringState {
            quote,
            escape: false,
            buf: String::new(),
        }
    }
}

pub fn parse_last(line: &str) -> Token {
    let mut token = Token::Empty;

    for c in line.chars() {
        match &mut token {
            Token::Empty => {
                match c {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        token = Token::Name(c.to_string());
                    },
                    '0'..='9' => {
                        token = Token::Value(c.to_string());
                    },
                    '\'' | '"' => {
                        token = Token::InString(StringState::new(c));
                    },
                    // ignore operators
                    _ => (),
                }
            },
            Token::Name(s) => {
                match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                        s.push(c);
                    },
                    // ignore operators
                    _ => {
                        token = Token::Empty;
                    },
                }
            },
            Token::InString(s) => {
                if s.escape {
                    s.buf.push(c);
                    s.escape = false;
                } else if c == '\\' {
                    s.escape = true;
                } else if c == s.quote {
                    // done
                    token = Token::Empty;
                } else {
                    s.buf.push(c);
                }
            },
            Token::Value(s) => {
                match c {
                    '0'..='9' => {
                        s.push(c);
                    },
                    // ignore operators
                    // a name can't directly follow a value
                    _ => {
                        token = Token::Empty;
                    },
                }
            },
        }
    }

    token
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_empty() {
        let token = parse_last("");
        assert_eq!(token, Token::Empty);
    }

    #[test]
    pub fn test_abc() {
        let token = parse_last("abc");
        assert_eq!(token, Token::Name("abc".into()));
    }

    #[test]
    pub fn test_in_string() {
        let token = parse_last("return url_encode(\"asdf");
        assert_eq!(token, Token::InString(StringState {
            quote: '"',
            escape: false,
            buf: "asdf".into(),
        }));
    }

    #[test]
    pub fn test_in_func() {
        let token = parse_last("return url_encode(\"asdf\") .. url_deco");
        assert_eq!(token, Token::Name("url_deco".into()));
    }
}
