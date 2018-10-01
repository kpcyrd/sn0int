#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;

pub mod errors;
pub use errors::*;


#[derive(Debug, Serialize, Deserialize)]
pub enum ApiResponse<T> {
    #[serde(rename="success")]
    Success(T),
    #[serde(rename="error")]
    Error(String),
}

impl<T> ApiResponse<T> {
    pub fn success(self) -> Result<T> {
        match self {
            ApiResponse::Success(x) => Ok(x),
            ApiResponse::Error(err) => bail!("Api returned error: {:?}", err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoamiResponse {
    #[serde(rename="user")]
    pub user: String,
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
