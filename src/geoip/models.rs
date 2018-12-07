use crate::errors::*;
use maxminddb::geoip2;
use std::collections::BTreeMap;


fn from_geoip_model_names(names: Option<BTreeMap<String, String>>) -> Option<String> {
    let names = match names {
        Some(names) => names,
        _ => return None,
    };

    names.get("en")
        .map(|x| x.to_owned())
}

#[derive(Debug, Serialize)]
pub struct GeoLookup {
    pub continent: Option<String>,
    pub continent_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl From<geoip2::City> for GeoLookup {
    fn from(lookup: geoip2::City) -> GeoLookup {
        // parse maxminddb lookup
        let continent = match lookup.continent {
            Some(continent) => Continent::from_maxmind(continent),
            _ => None,
        };
        let country = match lookup.country {
            Some(country) => Country::from_maxmind(country),
            _ => None,
        };
        let city = match lookup.city {
            Some(city) => from_geoip_model_names(city.names),
            _ => None,
        };
        let location = match lookup.location {
            Some(location) => Location::from_maxmind(&location),
            _ => None,
        };

        // flatten datastructure
        let (continent, continent_code) = match continent {
            Some(x) => (Some(x.name), Some(x.code)),
            _ => (None, None),
        };
        let (country, country_code) = match country {
            Some(x) => (Some(x.name), Some(x.code)),
            _ => (None, None),
        };
        let (latitude, longitude) = match location {
            Some(x) => (Some(x.latitude), Some(x.longitude)),
            _ => (None, None),
        };

        // return result
        GeoLookup {
            continent,
            continent_code,
            country,
            country_code,
            city,
            latitude,
            longitude,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Continent {
    code: String,
    name: String,
}

impl Continent {
    fn from_maxmind(continent: geoip2::model::Continent) -> Option<Self> {
        let code = match continent.code {
            Some(code) => code,
            _ => return None,
        };
        let name = match from_geoip_model_names(continent.names) {
            Some(name) => name,
            _ => return None,
        };

        Some(Continent {
            code,
            name,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Country {
    code: String,
    name: String,
}

impl Country {
    fn from_maxmind(country: geoip2::model::Country) -> Option<Self> {
        let code = match country.iso_code {
            Some(code) => code,
            _ => return None,
        };
        let name = match from_geoip_model_names(country.names) {
            Some(name) => name,
            _ => return None,
        };

        Some(Country {
            code,
            name,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    fn from_maxmind(location: &geoip2::model::Location) -> Option<Self> {
        let latitude = match location.latitude {
            Some(latitude) => latitude,
            _ => return None,
        };
        let longitude = match location.longitude {
            Some(longitude) => longitude,
            _ => return None,
        };

        Some(Location {
            latitude,
            longitude,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AsnLookup {
    asn: u32,
    as_org: String,
}

impl AsnLookup {
    pub fn try_from(lookup: geoip2::Isp) -> Result<AsnLookup> {
        // parse maxminddb lookup
        let asn = match lookup.autonomous_system_number {
            Some(asn) => asn,
            _ => bail!("autonomous_system_number not set"),
        };
        let as_org = match lookup.autonomous_system_organization {
            Some(org) => org,
            _ => bail!("autonomous_system_organization not set"),
        };

        // return result
        Ok(AsnLookup {
            asn,
            as_org,
        })
    }
}
