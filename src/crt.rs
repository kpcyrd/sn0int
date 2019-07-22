use crate::errors::*;

use x509_parser;
use der_parser::der::DerObject;
use der_parser::ber::{BerObjectContent, BerTag};
use der_parser::oid::Oid;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};


#[derive(Debug, PartialEq)]
pub enum AlternativeName {
    DnsName(String),
    IpAddr(IpAddr),
}

pub fn san_extension(i: &[u8]) -> Result<Vec<AlternativeName>> {
    let (rem, seq) = der_parser::der::parse_der_sequence(i)
        .map_err(|_| format_err!("Failed to parse san extension"))?;

    if !rem.is_empty() {
        bail!("san extension has trailing garbage");
    }

    debug!("Decoded sequence: {:?}", seq);
    if let BerObjectContent::Sequence(seq) = seq.content {
        seq.into_iter()
            .map(san_value)
            .collect()
    } else {
        bail!("Expected der sequence");
    }
}

pub fn san_value(o: DerObject) -> Result<AlternativeName> {
    debug!("DER object in SAN extension: {:?}", o);

    match (o.class, o.tag, &o.content) {
        (2, BerTag::Integer, BerObjectContent::Unknown(BerTag::Integer, value)) => san_value_dns(value),
        (2, BerTag::ObjDescriptor, BerObjectContent::Unknown(BerTag::ObjDescriptor, value)) => san_value_ipaddr(value),
        _ => bail!("Unexpected object: {:?}", o),
    }
}

pub fn san_value_dns(v: &[u8]) -> Result<AlternativeName> {
    debug!("Reading as dns name: {:?}", v);
    String::from_utf8(v.to_vec())
        .map(AlternativeName::DnsName)
        .map_err(Error::from)
}

pub fn san_value_ipaddr(v: &[u8]) -> Result<AlternativeName> {
    debug!("Reading as ipaddr: {:?}", v);
    match v.len() {
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
}

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
            let values = san_extension(x.value)?;

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
    use der_parser::parse_der;

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
        let ext = san_extension(&[48, 28,
                130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109,
                130, 14, 119, 119, 119, 46, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109
            ])
            .expect("Failed to parse extension");
        assert_eq!(ext, vec![
            AlternativeName::DnsName(String::from("github.com")),
            AlternativeName::DnsName(String::from("www.github.com")),
        ]);
    }

    #[test]
    fn test_long_san_extension() {
        let mut x = Certificate::parse_pem(r#"-----BEGIN CERTIFICATE-----
MIII3jCCB8agAwIBAgIQAp1dOviF3mpYKKObx4fjxjANBgkqhkiG9w0BAQsFADBe
MQswCQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3
d3cuZGlnaWNlcnQuY29tMR0wGwYDVQQDExRHZW9UcnVzdCBSU0EgQ0EgMjAxODAe
Fw0xOTAxMDMwMDAwMDBaFw0xOTA3MzAxMjAwMDBaMIGCMQswCQYDVQQGEwJERTEl
MCMGA1UECBMcRnJlaWUgdW5kIEhhbnNlc3RhZHQgSGFtYnVyZzEQMA4GA1UEBxMH
SGFtYnVyZzEXMBUGA1UEChMOQWJvdXQgWW91IEdtYkgxCzAJBgNVBAsTAklUMRQw
EgYDVQQDEwthYm91dHlvdS5kZTCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoC
ggEBALG6ZjY9TtJmN18p5KlJMtzdZMhw3mz6dGOYoSMTaQCDnw7RW14H8JX9Dz51
dTM4Ig1rPka5DjujNG8BKETGknRzQMEo7x08qZirzdQIz9QCnYDQ3/6l9tDfQ16X
pctnQRY156H8jyjhkaT+dWJIHaPwz+g6117plfv0F6iOcupNtF4rnZK7vpcyb/Fm
F985uHdBVXJKVt7BMUjUO6fdm8865fTyL8lb1ocEgbN91KdI7Bt9wUqxgOR7BJRJ
YQAC+Y6wqE8BwOGH11QaNGKQ8xGdBd3eC4tAuif1y+4WVPDAlhmJJR/FcnsiLVbX
zg4sgE+4kLOayCJY6MfN2MRtchkCAwEAAaOCBXEwggVtMB8GA1UdIwQYMBaAFJBY
/7CcdahRVHex7fKjQxY4nmzFMB0GA1UdDgQWBBTPNzAGXJdERAKW8w3kFNJ88MLO
qjCCAuIGA1UdEQSCAtkwggLVggthYm91dHlvdS5kZYIRY2RuLnlvdWFuZGlkb2wu
ZGWCDWNkbi5lZGl0ZWQuZGWCE2Nkbi5hYm91dHN0YXRpYy5jb22CDW0uYWJvdXR5
b3UuZGWCE3N0YXRpYzMuYWJvdXR5b3UuZGWCFmltYWdlcy5hYm91dHN0YXRpYy5j
b22CE3N0YXRpYzUuYWJvdXR5b3UuZGWCEGNvLXQuYWJvdXR5b3UuZGWCDXQuYWJv
dXR5b3UuZGWCDmNvLmFib3V0eW91LmRlggllZGl0ZWQuZGWCE2NvLW1hcHAuYWJv
dXR5b3UuZGWCE3N0YXRpYzQuYWJvdXR5b3UuZGWCEW1lZGlhLmFib3V0eW91LmRl
giZ3aXR0LXdlaWRlbi5kYW0uc3RhZ2luZy5hYm91dHlvdS5jbG91ZIISc3RhdGlj
LmFib3V0eW91LmRlghBjZG40LmFib3V0eW91LmRlghNzdGF0aWMyLmFib3V0eW91
LmRlgiN3aXR0LXdlaWRlbi5kYW0uYWNtZS5hYm91dHlvdS5jbG91ZIIXY2RuLmFi
b3V0eW91LXN0YWdpbmcuZGWCD2Nkbi5hYm91dHlvdS5kZYIQY2RuMy5hYm91dHlv
dS5kZYIQY2RuMi5hYm91dHlvdS5kZYIQY2RuNS5hYm91dHlvdS5kZYISYXNzZXRz
LmFib3V0eW91LmRlghBjZG4xLmFib3V0eW91LmRlghpzdGF0aWNtYWlsLWNkbi5h
Ym91dHlvdS5kZYIPd3d3LmFib3V0eW91LmRlghNzdGF0aWMxLmFib3V0eW91LmRl
ghRtLWFzc2V0cy5hYm91dHlvdS5kZYIQY2RuLm1hcnktcGF1bC5kZYIQY28tbS5h
Ym91dHlvdS5kZYIVZmlsZXMuYWJvdXRzdGF0aWMuY29tghNpbWcuYWJvdXRzdGF0
aWMuY29tgg9pbWcuYWJvdXR5b3UuZGUwDgYDVR0PAQH/BAQDAgWgMB0GA1UdJQQW
MBQGCCsGAQUFBwMBBggrBgEFBQcDAjA+BgNVHR8ENzA1MDOgMaAvhi1odHRwOi8v
Y2RwLmdlb3RydXN0LmNvbS9HZW9UcnVzdFJTQUNBMjAxOC5jcmwwTAYDVR0gBEUw
QzA3BglghkgBhv1sAQEwKjAoBggrBgEFBQcCARYcaHR0cHM6Ly93d3cuZGlnaWNl
cnQuY29tL0NQUzAIBgZngQwBAgIwdQYIKwYBBQUHAQEEaTBnMCYGCCsGAQUFBzAB
hhpodHRwOi8vc3RhdHVzLmdlb3RydXN0LmNvbTA9BggrBgEFBQcwAoYxaHR0cDov
L2NhY2VydHMuZ2VvdHJ1c3QuY29tL0dlb1RydXN0UlNBQ0EyMDE4LmNydDAJBgNV
HRMEAjAAMIIBBAYKKwYBBAHWeQIEAgSB9QSB8gDwAHcAY/Lbzeg7zCzPC3KEJ1dr
M6SNYXePvXWmOLHHaFRL2I0AAAFoE/H84wAABAMASDBGAiEAh8Q7LXUzhsbiuxCS
VoeRmnPtLEZcjNFg3R+eBK5FkQMCIQD+Ic1QErzzP1B76BPLcaBgOxULpLQ2Ib4M
b38fMU5GhwB1AId1v+dZfPiMQ5lfvfNu/1aNR1Y2/0q1YMG06v9eoIMPAAABaBPx
/dQAAAQDAEYwRAIgDcWLzLdGGG7d3EV3y809H8MwEojfEXT0DS75TchCvB0CIBno
kC5/KGjNdQdsqJX4NJbQ06RAbHLeGwX5ccmaKbQ3MA0GCSqGSIb3DQEBCwUAA4IB
AQC81DWjm2PklQzIGSIf/tRm2GtjlL6Vi7rMGkSbiV0k1FnoptdHfQIs55tTBD7c
TheMOk62JL6z0FKpAgPUIU+HrKJ/fAcBmQo+yqn0vRT0yhDrDGEFl6Sm2HyI0oKG
XryhpFLQkHuDkyA4uKOLuefPBgdjVZW9LqxmZhFPaZY6BSa/neZopVNwC1c+4Xwu
mAlnYDoB0Mj2UIPvIeftkDfF6sURmmZb0/+AMbFDCQYHvZFPI8DFgcagy8og5XJZ
gQ+70UdJdM3RWyrd9R66aZwNGkcS6C2wtKCRhztWDMru/wNuyOsYS6JttoTYxRsh
z/6Vy8Ga9kigYVsa8ZFMR+Ex
-----END CERTIFICATE-----
"#).expect("Failed to parse cert");
        x.valid_names.sort();
        x.valid_ipaddrs.sort();
        assert_eq!(x, Certificate {
            valid_names: vec![
                "aboutyou.de".into(),
                "assets.aboutyou.de".into(),
                "cdn.aboutstatic.com".into(),
                "cdn.aboutyou-staging.de".into(),
                "cdn.aboutyou.de".into(),
                "cdn.edited.de".into(),
                "cdn.mary-paul.de".into(),
                "cdn.youandidol.de".into(),
                "cdn1.aboutyou.de".into(),
                "cdn2.aboutyou.de".into(),
                "cdn3.aboutyou.de".into(),
                "cdn4.aboutyou.de".into(),
                "cdn5.aboutyou.de".into(),
                "co-m.aboutyou.de".into(),
                "co-mapp.aboutyou.de".into(),
                "co-t.aboutyou.de".into(),
                "co.aboutyou.de".into(),
                "edited.de".into(),
                "files.aboutstatic.com".into(),
                "images.aboutstatic.com".into(),
                "img.aboutstatic.com".into(),
                "img.aboutyou.de".into(),
                "m-assets.aboutyou.de".into(),
                "m.aboutyou.de".into(),
                "media.aboutyou.de".into(),
                "static.aboutyou.de".into(),
                "static1.aboutyou.de".into(),
                "static2.aboutyou.de".into(),
                "static3.aboutyou.de".into(),
                "static4.aboutyou.de".into(),
                "static5.aboutyou.de".into(),
                "staticmail-cdn.aboutyou.de".into(),
                "t.aboutyou.de".into(),
                "witt-weiden.dam.acme.aboutyou.cloud".into(),
                "witt-weiden.dam.staging.aboutyou.cloud".into(),
                "www.aboutyou.de".into(),
            ],
            valid_ipaddrs: vec![],
        });
    }

    #[test]
    fn test_san_value_dns() {
        let (rem, v) = parse_der(&[130, 10, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109])
            .expect("Failed to parse san value");
        assert!(rem.is_empty());
        println!("{:?}", v);
        assert_eq!(v, DerObject {
            class: 2,
            structured: 0,
            tag: BerTag::Integer,
            content: BerObjectContent::Unknown(BerTag::Integer, &[103, 105, 116, 104, 117, 98, 46, 99, 111, 109])
        });
        let content = match v.content {
            BerObjectContent::Unknown(BerTag::Integer, v) => v,
            _ => panic!("Wrong BerObjectContent"),
        };
        let v = san_value_dns(content)
            .expect("Failed to process san value");
        assert_eq!(v, AlternativeName::DnsName(String::from("github.com")));
    }

    #[test]
    fn test_san_value_ipaddr() {
        let (rem, v) = parse_der(&[135, 4, 1, 1, 1, 1])
            .expect("Failed to parse san value");
        assert!(rem.is_empty());
        println!("{:?}", v);
        assert_eq!(v, DerObject {
            class: 2,
            structured: 0,
            tag: BerTag::ObjDescriptor,
            content: BerObjectContent::Unknown(BerTag::ObjDescriptor, &[1, 1, 1, 1])
        });
        let content = match v.content {
            BerObjectContent::Unknown(BerTag::ObjDescriptor, v) => v,
            _ => panic!("Wrong BerObjectContent"),
        };
        let v = san_value_ipaddr(content)
            .expect("Failed to process san value");
        assert_eq!(v, AlternativeName::IpAddr("1.1.1.1".parse().unwrap()));
    }
}
