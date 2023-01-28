use hlua_badtouch as hlua;

pub mod blobs;
pub mod crt;
pub mod crypto;
pub mod engine;
mod errors;
pub mod geo;
pub mod geoip;
pub mod gfx;
pub mod html;
pub mod json;
pub mod lazy;
pub mod mqtt;
pub mod psl;
pub mod ratelimits;
pub mod sockets;
pub mod web;
pub mod websockets;
pub mod xml;

#[cfg(test)]
fn test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
