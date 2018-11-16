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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

    #[test]
    fn test_parse_pem() {
        let mut x = Certificate::parse_pem(r#"-----BEGIN CERTIFICATE-----
MIIHQjCCBiqgAwIBAgIQCgYwQn9bvO1pVzllk7ZFHzANBgkqhkiG9w0BAQsFADB1
MQswCQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3
d3cuZGlnaWNlcnQuY29tMTQwMgYDVQQDEytEaWdpQ2VydCBTSEEyIEV4dGVuZGVk
IFZhbGlkYXRpb24gU2VydmVyIENBMB4XDTE4MDUwODAwMDAwMFoXDTIwMDYwMzEy
MDAwMFowgccxHTAbBgNVBA8MFFByaXZhdGUgT3JnYW5pemF0aW9uMRMwEQYLKwYB
BAGCNzwCAQMTAlVTMRkwFwYLKwYBBAGCNzwCAQITCERlbGF3YXJlMRAwDgYDVQQF
Ewc1MTU3NTUwMQswCQYDVQQGEwJVUzETMBEGA1UECBMKQ2FsaWZvcm5pYTEWMBQG
A1UEBxMNU2FuIEZyYW5jaXNjbzEVMBMGA1UEChMMR2l0SHViLCBJbmMuMRMwEQYD
VQQDEwpnaXRodWIuY29tMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA
xjyq8jyXDDrBTyitcnB90865tWBzpHSbindG/XqYQkzFMBlXmqkzC+FdTRBYyneZ
w5Pz+XWQvL+74JW6LsWNc2EF0xCEqLOJuC9zjPAqbr7uroNLghGxYf13YdqbG5oj
/4x+ogEG3dF/U5YIwVr658DKyESMV6eoYV9mDVfTuJastkqcwero+5ZAKfYVMLUE
sMwFtoTDJFmVf6JlkOWwsxp1WcQ/MRQK1cyqOoUFUgYylgdh3yeCDPeF22Ax8AlQ
xbcaI+GwfQL1FB7Jy+h+KjME9lE/UpgV6Qt2R1xNSmvFCBWu+NFX6epwFP/JRbkM
fLz0beYFUvmMgLtwVpEPSwIDAQABo4IDeTCCA3UwHwYDVR0jBBgwFoAUPdNQpdag
re7zSmAKZdMh1Pj41g8wHQYDVR0OBBYEFMnCU2FmnV+rJfQmzQ84mqhJ6kipMCUG
A1UdEQQeMByCCmdpdGh1Yi5jb22CDnd3dy5naXRodWIuY29tMA4GA1UdDwEB/wQE
AwIFoDAdBgNVHSUEFjAUBggrBgEFBQcDAQYIKwYBBQUHAwIwdQYDVR0fBG4wbDA0
oDKgMIYuaHR0cDovL2NybDMuZGlnaWNlcnQuY29tL3NoYTItZXYtc2VydmVyLWcy
LmNybDA0oDKgMIYuaHR0cDovL2NybDQuZGlnaWNlcnQuY29tL3NoYTItZXYtc2Vy
dmVyLWcyLmNybDBLBgNVHSAERDBCMDcGCWCGSAGG/WwCATAqMCgGCCsGAQUFBwIB
FhxodHRwczovL3d3dy5kaWdpY2VydC5jb20vQ1BTMAcGBWeBDAEBMIGIBggrBgEF
BQcBAQR8MHowJAYIKwYBBQUHMAGGGGh0dHA6Ly9vY3NwLmRpZ2ljZXJ0LmNvbTBS
BggrBgEFBQcwAoZGaHR0cDovL2NhY2VydHMuZGlnaWNlcnQuY29tL0RpZ2lDZXJ0
U0hBMkV4dGVuZGVkVmFsaWRhdGlvblNlcnZlckNBLmNydDAMBgNVHRMBAf8EAjAA
MIIBfgYKKwYBBAHWeQIEAgSCAW4EggFqAWgAdgCkuQmQtBhYFIe7E6LMZ3AKPDWY
BPkb37jjd80OyA3cEAAAAWNBYm0KAAAEAwBHMEUCIQDRZp38cTWsWH2GdBpe/uPT
Wnsu/m4BEC2+dIcvSykZYgIgCP5gGv6yzaazxBK2NwGdmmyuEFNSg2pARbMJlUFg
U5UAdgBWFAaaL9fC7NP14b1Esj7HRna5vJkRXMDvlJhV1onQ3QAAAWNBYm0tAAAE
AwBHMEUCIQCi7omUvYLm0b2LobtEeRAYnlIo7n6JxbYdrtYdmPUWJQIgVgw1AZ51
vK9ENinBg22FPxb82TvNDO05T17hxXRC2IYAdgC72d+8H4pxtZOUI5eqkntHOFeV
CqtS6BqQlmQ2jh7RhQAAAWNBYm3fAAAEAwBHMEUCIQChzdTKUU2N+XcqcK0OJYrN
8EYynloVxho4yPk6Dq3EPgIgdNH5u8rC3UcslQV4B9o0a0w204omDREGKTVuEpxG
eOQwDQYJKoZIhvcNAQELBQADggEBAHAPWpanWOW/ip2oJ5grAH8mqQfaunuCVE+v
ac+88lkDK/LVdFgl2B6kIHZiYClzKtfczG93hWvKbST4NRNHP9LiaQqdNC17e5vN
HnXVUGw+yxyjMLGqkgepOnZ2Rb14kcTOGp4i5AuJuuaMwXmCo7jUwPwfLe1NUlVB
Kqg6LK0Hcq4K0sZnxE8HFxiZ92WpV2AVWjRMEc/2z2shNoDvxvFUYyY1Oe67xINk
myQKc+ygSBZzyLnXSFVWmHr3u5dcaaQGGAR42v6Ydr4iL38Hd4dOiBma+FXsXBIq
WUjbST4VXmdaol7uzFMojA4zkxQDZAvF5XgJlAFadfySna/teik=
-----END CERTIFICATE-----
"#).expect("Failed to parse cert");
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
