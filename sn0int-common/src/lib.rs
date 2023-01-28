pub mod api;
pub mod errors;
pub use crate::errors::*;
pub mod id;
pub mod metadata;
pub use crate::id::*;

pub use rocket_failure_errors::StrictApiResponse as ApiResponse;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
