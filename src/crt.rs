use crate::errors::*;

use x509_parser;
use der_parser::oid::Oid;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use nom::be_u8;


#[derive(Debug, PartialEq)]
pub enum AlternativeName {
    DnsName(String),
    IpAddr(IpAddr),
}

named!(san_extension<&[u8], Vec<AlternativeName>>, do_parse!(
    _tag:   tag!(b"\x30")   >>
    len:    be_u8           >>
    values: take!(len)      >>
    ({
        let mut bytes = values;
        let mut values = Vec::new();
        while !bytes.is_empty() {
            let (rem, v) = san_value(bytes)?;
            match v {
                Ok(v) => values.push(v),
                Err(e) => warn!("Unknown field in SAN extension: {}", e),
            }
            bytes = rem;
        }
        values
    })
));

named!(san_value<&[u8], Result<AlternativeName>>,
    switch!(be_u8,
        0x82    => call!(san_value_dns) |
        0x87    => call!(san_value_ipaddr) |
        k       => call!(san_value_unknown, k)
    )
);

named!(san_value_dns<&[u8], Result<AlternativeName>>, do_parse!(
    len:    be_u8           >>
    value:  take!(len)      >>
    ({
        String::from_utf8(value.to_vec())
            .map(AlternativeName::DnsName)
            .map_err(Error::from)
    })
));

named!(san_value_ipaddr<&[u8], Result<AlternativeName>>, do_parse!(
    len:    be_u8           >>
    v:      take!(len)      >>
    ({
        match len {
            4  => Ok(AlternativeName::IpAddr(Ipv4Addr::from([
                v[0], v[1], v[2], v[3],
            ]).into())),
            16 => Ok(AlternativeName::IpAddr(Ipv6Addr::from([
                v[0],  v[1],  v[2],  v[3],
                v[4],  v[5],  v[6],  v[7],
                v[8],  v[9],  v[10], v[11],
                v[12], v[13], v[14], v[15],
            ]).into())),
            _ => Err(format_err!("Invalid ipaddr")),
        }
    })
));

named_args!(san_value_unknown(key: u8)<&[u8], Result<AlternativeName>>, do_parse!(
    len:    be_u8           >>
    v:      take!(len)      >>
    ({
        Err(format_err!("Unexpected type {:?} => {:?}", key, v))
    })
));

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Certificate {
    pub valid_names: Vec<String>,
    pub valid_ipaddrs: Vec<IpAddr>
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
        let mut valid_ipaddrs = HashSet::new();

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

            debug!("Found san extension: {:?}", x.value);
            let values = match san_extension(x.value) {
                Ok((remaining, values)) => {
                    if !remaining.is_empty() {
                        bail!("san extension has trailing garbage");
                    }
                    values
                },
                Err(_) => bail!("Failed to parse san extension"),
            };

            for v in values {
                match v {
                    AlternativeName::DnsName(v) => valid_names.insert(v),
                    AlternativeName::IpAddr(v) => valid_ipaddrs.insert(v),
                };
            }
        }

        let valid_names = valid_names.into_iter().collect();
        let valid_ipaddrs = valid_ipaddrs.into_iter().collect();
        Ok(Certificate {
            valid_names,
            valid_ipaddrs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pem_github() {
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
            valid_ipaddrs: vec![],
        });
    }

    #[test]
    fn test_parse_pem_1_1_1_1() {
        let mut x = Certificate::parse_pem(r#"-----BEGIN CERTIFICATE-----
MIID9DCCA3qgAwIBAgIQBWzetBRl/ycHFsBukRYuGTAKBggqhkjOPQQDAjBMMQsw
CQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMSYwJAYDVQQDEx1EaWdp
Q2VydCBFQ0MgU2VjdXJlIFNlcnZlciBDQTAeFw0xODAzMzAwMDAwMDBaFw0yMDAz
MjUxMjAwMDBaMGwxCzAJBgNVBAYTAlVTMQswCQYDVQQIEwJDQTEWMBQGA1UEBxMN
U2FuIEZyYW5jaXNjbzEZMBcGA1UEChMQQ2xvdWRmbGFyZSwgSW5jLjEdMBsGA1UE
AwwUKi5jbG91ZGZsYXJlLWRucy5jb20wWTATBgcqhkjOPQIBBggqhkjOPQMBBwNC
AASyRQsxrFBjziHmfDQjGsXBU0WWl3oxh7vg6h2V9f8lBMp18PY/td9R6VvJPa20
AwVzIJI+dL6OSxviaIZEbmK7o4ICHDCCAhgwHwYDVR0jBBgwFoAUo53mH/naOU/A
buiRy5Wl2jHiCp8wHQYDVR0OBBYEFN+XTeVDs7BBp0LykM+Jf64SV4ThMGMGA1Ud
EQRcMFqCFCouY2xvdWRmbGFyZS1kbnMuY29thwQBAQEBhwQBAAABghJjbG91ZGZs
YXJlLWRucy5jb22HECYGRwBHAAAAAAAAAAAAERGHECYGRwBHAAAAAAAAAAAAEAEw
DgYDVR0PAQH/BAQDAgeAMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcDAjBp
BgNVHR8EYjBgMC6gLKAqhihodHRwOi8vY3JsMy5kaWdpY2VydC5jb20vc3NjYS1l
Y2MtZzEuY3JsMC6gLKAqhihodHRwOi8vY3JsNC5kaWdpY2VydC5jb20vc3NjYS1l
Y2MtZzEuY3JsMEwGA1UdIARFMEMwNwYJYIZIAYb9bAEBMCowKAYIKwYBBQUHAgEW
HGh0dHBzOi8vd3d3LmRpZ2ljZXJ0LmNvbS9DUFMwCAYGZ4EMAQICMHsGCCsGAQUF
BwEBBG8wbTAkBggrBgEFBQcwAYYYaHR0cDovL29jc3AuZGlnaWNlcnQuY29tMEUG
CCsGAQUFBzAChjlodHRwOi8vY2FjZXJ0cy5kaWdpY2VydC5jb20vRGlnaUNlcnRF
Q0NTZWN1cmVTZXJ2ZXJDQS5jcnQwDAYDVR0TAQH/BAIwADAKBggqhkjOPQQDAgNo
ADBlAjEAjoyy2Ogh1i1/Kh9+psMc1OChlQIvQF6AkojZS8yliar6m8q5nqC3qe0h
HR0fExwLAjAueWRnHX4QJ9loqMhsPk3NB0Cs0mStsNDNG6/DpCYw7XmjoG3y1LS7
ZkZZmqNn2Q8=
-----END CERTIFICATE-----
"#).expect("Failed to parse cert");
        x.valid_names.sort();
        x.valid_ipaddrs.sort();
        assert_eq!(x, Certificate {
            valid_names: vec![
                "*.cloudflare-dns.com".into(),
                "cloudflare-dns.com".into(),
            ],
            valid_ipaddrs: vec![
                "1.0.0.1".parse().unwrap(),
                "1.1.1.1".parse().unwrap(),
                "2606:4700:4700::1001".parse().unwrap(),
                "2606:4700:4700::1111".parse().unwrap(),
            ],
        });
    }

    #[test]
    fn test_san_extension() {
        let (rem, ext) = san_extension(&[48, 28,
                130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109,
                130, 14, 119, 119, 119, 46, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109
            ])
            .expect("Failed to parse extension");
        assert!(rem.is_empty());
        assert_eq!(ext, vec![
            AlternativeName::DnsName(String::from("github.com")),
            AlternativeName::DnsName(String::from("www.github.com")),
        ]);
    }

    #[test]
    fn test_san_value_dns() {
        let (rem, v) = san_value(&[130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109])
            .expect("Failed to parse san value");
        let v = v.expect("Extension contains invalid data");
        assert!(rem.is_empty());
        assert_eq!(v, AlternativeName::DnsName(String::from("github.com")));
    }

    #[test]
    fn test_san_value_ipaddr() {
        let (rem, v) = san_value(&[135, 4, 1, 1, 1, 1])
            .expect("Failed to parse san value");
        let v = v.expect("Extension contains invalid data");
        assert!(rem.is_empty());
        assert_eq!(v, AlternativeName::IpAddr("1.1.1.1".parse().unwrap()));
    }
}
