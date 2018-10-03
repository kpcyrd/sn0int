#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;
#[macro_use] extern crate nom;

pub mod api;
pub mod errors;
pub use errors::*;
pub mod metadata;
pub mod id;
pub use id::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
