use errors::*;

use x509_parser;
use der_parser::oid::Oid;
use std::collections::HashSet;
use nom::be_u8;


named!(san_extension<&[u8], Result<Vec<String>>>, do_parse!(
    _tag:   tag!(b"\x30")   >>
    len:    be_u8           >>
    values: take!(len)      >>
    ({
        let mut bytes = values;
        let mut values = Vec::new();
        while !bytes.is_empty() {
            let (rem, v) = san_value(bytes)?;
            values.push(v);
            bytes = rem;
        }

        values.into_iter()
            .collect()
    })
));

named!(san_value<&[u8], Result<String>>, do_parse!(
    _tag:   tag!(b"\x82")   >>
    len:    be_u8           >>
    value:  take!(len)      >>
    ({
        String::from_utf8(value.to_vec())
            .map_err(Error::from)
    })
));

#[derive(Debug, PartialEq)]
pub struct Certificate {
    pub valid_names: Vec<String>,
}

impl Certificate {
    pub fn parse_pem(crt: &str) -> Result<Certificate> {
        let pem = match x509_parser::pem::pem_to_der(crt.as_bytes()) {
            Ok((remaining, pem)) => {
                if !remaining.is_empty() {
                    bail!("input cert has trailing garbage");
                }
                if pem.label != "CERTIFICATE" {
                    bail!("input is not a certificate");
                }
                pem
            },
            Err(_) => bail!("Failed to parse pem"),
        };
        Certificate::from_bytes(&pem.contents)
    }

    pub fn from_bytes(crt: &[u8]) -> Result<Certificate> {
        let crt = match x509_parser::parse_x509_der(&crt) {
            Ok((remaining, der)) => {
                if !remaining.is_empty() {
                    bail!("input cert has trailing garbage");
                }
                if der.tbs_certificate.version != 2 {
                    bail!("unexpected certificate version");
                }
                der
            },
            Err(_) => bail!("Failed to parse der"),
        };

        let mut valid_names = HashSet::new();

        for x in crt.tbs_certificate.subject.rdn_seq {
            for y in x.set {
                // CommonName
                if y.attr_type != Oid::from(&[2, 5, 4, 3]) {
                    continue;
                }

                if let Ok(x) = y.attr_value.content.as_slice() {
                    let value = String::from_utf8(x.to_vec())?;
                    info!("Found CN in Subject: {:?}", value);
                    valid_names.insert(value);
                }
            }
        }

        for x in crt.tbs_certificate.extensions {
            if x.oid != Oid::from(&[2, 5, 29, 17]) {
                continue;
            }

            let values = match san_extension(x.value) {
                Ok((remaining, values)) => {
                    if !remaining.is_empty() {
                        bail!("san extension has trailing garbage");
                    }
                    values?
                },
                Err(_) => bail!("Failed to parse san extension"),
            };

            valid_names.extend(values);
        }

        let valid_names = valid_names.into_iter().collect();
        Ok(Certificate {
            valid_names,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GITHUB_PEM: &str = include_str!("../ci/github.pem");

    #[test]
    fn test_parse_pem() {
        let mut x = Certificate::parse_pem(GITHUB_PEM).expect("Failed to parse cert");
        x.valid_names.sort();
        assert_eq!(x, Certificate {
            valid_names: vec!["github.com".into(), "www.github.com".into()],
        });
    }

    #[test]
    fn test_san_extension() {
        let (rem, ext) = san_extension(&[48, 28,
                130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109,
                130, 14, 119, 119, 119, 46, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109
            ])
            .expect("Failed to parse extension");
        let ext = ext.expect("Extension contains invalid data");
        assert!(rem.is_empty());
        assert_eq!(ext, vec![
            String::from("github.com"),
            String::from("www.github.com"),
        ]);

    }

    #[test]
    fn test_san_value() {
        let (rem, v) = san_value(&[130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109])
            .expect("Failed to parse san value");
        let v = v.expect("Extension contains invalid data");
        assert!(rem.is_empty());
        assert_eq!(v, String::from("github.com"));
    }
}
