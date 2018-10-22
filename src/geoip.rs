use errors::*;
use maxminddb::{self, geoip2};
use std::net::IpAddr;
use std::collections::BTreeMap;

pub static PATH: &str = "/usr/share/GeoIP/GeoLite2-City.mmdb";


fn from_geoip_model_names(names: Option<BTreeMap<String, String>>) -> Option<String> {
    let names = match names {
        Some(names) => names,
        _ => return None,
    };

    names.get("en")
        .map(|x| x.to_owned())
}

#[derive(Debug)]
pub struct Lookup {
    continent: Option<Continent>,
    country: Option<Country>,
    city: Option<String>,
    location: Option<Location>,
}

impl From<geoip2::City> for Lookup {
    fn from(lookup: geoip2::City) -> Lookup {
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
            Some(location) => Location::from_maxmind(location),
            _ => None,
        };

        Lookup {
            continent,
            country,
            city,
            location,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    fn from_maxmind(location: geoip2::model::Location) -> Option<Self> {
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

pub fn lookup(ip: IpAddr) -> Result<Lookup> {
    let reader = maxminddb::Reader::open(PATH)
        .context("Failed to open geoip database")?;
    let city: geoip2::City = reader.lookup(ip)?;
    debug!("GeoIP result: {:?}", city);
    Ok(Lookup::from(city))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geoip_lookup() {
        let ip = "1.1.1.1".parse().unwrap();
        let lookup = lookup(ip).expect("GeoIP lookup failed");
        println!("{:#?}", lookup);
        assert_eq!(lookup.city, None);
    }
}
