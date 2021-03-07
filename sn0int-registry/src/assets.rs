use blake2::{Blake2b, Digest};
use lazy_static::lazy_static;

pub static FAVICON: &[u8] = include_bytes!("../assets/favicon.ico");
pub static STYLESHEET: &[u8] = include_bytes!("../assets/style.css");
pub static JAVASCRIPT: &[u8] = include_bytes!("../assets/script.js");
pub static SOCIAL_CARD: &[u8] = include_bytes!("../assets/social-card.png");

pub static JAVASCRIPT_CLIPBOARD: &[u8] = include_bytes!("../assets/clipboard.min.js");

pub static FA_STYLESHEET: &[u8] = include_bytes!("../assets/fontawesome/css/all.min.css");
pub static FA_BRANDS_400_SVG: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-brands-400.svg");
pub static FA_BRANDS_400_TTF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-brands-400.ttf");
pub static FA_BRANDS_400_WOFF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-brands-400.woff");
pub static FA_BRANDS_400_WOFF2: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-brands-400.woff2");
pub static FA_REGULAR_400_SVG: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-regular-400.svg");
pub static FA_REGULAR_400_TTF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-regular-400.ttf");
pub static FA_REGULAR_400_WOFF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-regular-400.woff");
pub static FA_REGULAR_400_WOFF2: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-regular-400.woff2");
pub static FA_SOLID_900_SVG: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-solid-900.svg");
pub static FA_SOLID_900_TTF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-solid-900.ttf");
pub static FA_SOLID_900_WOFF: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-solid-900.woff");
pub static FA_SOLID_900_WOFF2: &[u8] = include_bytes!("../assets/fontawesome/webfonts/fa-solid-900.woff2");

lazy_static! {
    pub static ref ASSET_REV: String = {
        let mut h = Blake2b::new();
        h.input(STYLESHEET);
        h.input(JAVASCRIPT);

        h.input(JAVASCRIPT_CLIPBOARD);

        h.input(FA_STYLESHEET);
        h.input(FA_BRANDS_400_SVG);
        h.input(FA_BRANDS_400_TTF);
        h.input(FA_BRANDS_400_WOFF);
        h.input(FA_BRANDS_400_WOFF2);
        h.input(FA_REGULAR_400_SVG);
        h.input(FA_REGULAR_400_TTF);
        h.input(FA_REGULAR_400_WOFF);
        h.input(FA_REGULAR_400_WOFF2);
        h.input(FA_SOLID_900_SVG);
        h.input(FA_SOLID_900_TTF);
        h.input(FA_SOLID_900_WOFF);
        h.input(FA_SOLID_900_WOFF2);

        hex::encode(&h.result()[0..8])
    };
}
