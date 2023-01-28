use crate::errors::*;
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Location {
    latitude: f64,
    longitude: f64,
}

impl Location {
    fn try_from_iter<'a, I: IntoIterator<Item=&'a exif::Field>>(iter: I) -> Result<Self> {
        let mut builder = LocationBuilder::default();
        for f in iter {
            debug!("Exif field: {:?}", f.display_value().to_string());
            builder.add_one(f)?;
        }
        builder.build()
    }
}

#[derive(Debug, Default)]
struct LocationBuilder {
    latitude: Option<f64>,
    latitude_ref: Option<f64>,
    longitude: Option<f64>,
    longitude_ref: Option<f64>,
}

impl LocationBuilder {
    fn add_one(&mut self, f: &exif::Field) -> Result<()> {
        debug!("Exif tag: {:?}, {}", f.tag, f.value.display_as(f.tag));
        match f.tag {
            exif::Tag::GPSLatitudeRef => self.latitude_ref = Some(cardinal_direction_modifier(&f.value)?),
            exif::Tag::GPSLongitudeRef => self.longitude_ref = Some(cardinal_direction_modifier(&f.value)?),
            exif::Tag::GPSLatitude => self.latitude = Some(dms_to_float(&f.value)?),
            exif::Tag::GPSLongitude => self.longitude = Some(dms_to_float(&f.value)?),
            _ => (),
        };
        Ok(())
    }

    fn build(self) -> Result<Location> {
        let latitude = self.latitude
            .ok_or_else(|| format_err!("Missing latitude field"))?;
        let latitude_ref = self.latitude_ref
            .ok_or_else(|| format_err!("Missing latitude field"))?;

        let longitude = self.longitude
            .ok_or_else(|| format_err!("Missing latitude field"))?;
        let longitude_ref = self.longitude_ref
            .ok_or_else(|| format_err!("Missing latitude field"))?;

        Ok(Location {
            latitude: latitude * latitude_ref,
            longitude: longitude * longitude_ref,
        })
    }
}

pub fn gps(img: &[u8]) -> Result<Option<Location>> {
    let mut buf = io::Cursor::new(img);
    let reader = exif::Reader::new()
        .read_from_container(&mut buf)?;
    let fields = reader.fields();

    let location = Location::try_from_iter(fields).ok();
    Ok(location)
}

pub fn dms_to_float(dms: &exif::Value) -> Result<f64> {
    let dms = match dms {
        exif::Value::Rational(dms) => dms,
        _ => panic!("Unexpected exif value for dms"),
    };

    if dms.len() != 3 {
        bail!("Incorrect numbers for floats for dms");
    }

    let degrees = dms[0].to_f64();
    let minutes = dms[1].to_f64();
    let seconds = dms[2].to_f64();

    let float = degrees + minutes/60.0 + seconds/3600.0;
    let float = (float * 1000000.0).round() / 1000000.0;
    Ok(float)
}

pub fn cardinal_direction_modifier(value: &exif::Value) -> Result<f64> {
    match value {
        exif::Value::Ascii(s) => {
            let s = s.get(0)
                .ok_or_else(|| format_err!("Cardinal direction value is empty"))?;

            match s.first() {
                Some(b'N') => Ok(1.0),
                Some(b'S') => Ok(-1.0),
                Some(b'E') => Ok(1.0),
                Some(b'W') => Ok(-1.0),
                _ => bail!("Unexpected cardinal direction"),
            }
        },
        _ => bail!("Unexpected exif value"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_init;

    #[test]
    fn verify_exif_location() {
        test_init();

        let location = Location::try_from_iter(&[
            exif::Field {
                tag: exif::Tag::GPSLatitudeRef,
                ifd_num: exif::In::PRIMARY,
                value: exif::Value::Ascii(vec![vec![b'N']]),
            }, exif::Field {
                tag: exif::Tag::GPSLongitudeRef,
                ifd_num: exif::In::PRIMARY,
                value: exif::Value::Ascii(vec![vec![b'E']]),
            }, exif::Field {
                tag: exif::Tag::GPSLatitude,
                ifd_num: exif::In::PRIMARY,
                value: exif::Value::Rational(vec![exif::Rational {
                    num: 43,
                    denom: 1,
                }, exif::Rational {
                    num: 28,
                    denom: 1,
                }, exif::Rational {
                    num: 176399999,
                    denom: 100000000,
                }]),
            }, exif::Field {
                tag: exif::Tag::GPSLongitude,
                ifd_num: exif::In::PRIMARY,
                value: exif::Value::Rational(vec![exif::Rational {
                    num: 11,
                    denom: 1,
                }, exif::Rational {
                    num: 53,
                    denom: 1,
                }, exif::Rational {
                    num: 742199999,
                    denom: 100000000,
                }]),
            },
        ]).unwrap();
        println!("{:?}", location);

        assert_eq!(location, Location {
            latitude: 43.467157,
            longitude: 11.885395
        });
    }

    #[test]
    fn verify_dms() {
        test_init();

        let latitude = dms_to_float(&exif::Value::Rational(vec![exif::Rational {
            num: 43,
            denom: 1,
        }, exif::Rational {
            num: 28,
            denom: 1,
        }, exif::Rational {
            num: 176399999,
            denom: 100000000,
        }])).unwrap();

        assert_eq!(latitude, 43.467157);
    }
}
