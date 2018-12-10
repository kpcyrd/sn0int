use blake2::{Blake2b, Digest};


pub static FAVICON: &[u8] = include_bytes!("../assets/favicon.ico");
pub static STYLE_SHEET: &[u8] = include_bytes!("../assets/style.css");

lazy_static! {
    pub static ref ASSET_REV: String = {
        let mut h = Blake2b::new();
        h.input(STYLE_SHEET);
        hex::encode(&h.result()[0..8])
    };
}
