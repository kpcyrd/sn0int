use errors::*;
use auth::Authenticator;
use auth2::AuthHeader;
use db;
use sn0int_common::{ApiResponse, WhoamiResponse};
use rocket::response::Redirect;
use rocket_contrib::{Json, Value};


#[get("/dashboard")]
fn dashboard() -> Json<Value> {
    Json(json!({ "dashboard": {}}))
}

#[derive(Debug, FromForm)]
pub struct Search {
    q: String,
}

#[get("/search?<q>")]
fn search(q: Search) -> Json<Value> {
    println!("{:?}", q);
    Json(json!({ "dashboard": {}}))
}

#[get("/dl/<author>/<name>", format="application/json")]
fn download(author: String, name: String) -> Json<Value> {
    println!("{:?}/{:?}", author, name);
    Json(json!({ "dashboard": {}}))
}

#[post("/publish/<author>/<name>", format="application/json", data="<upload>")]
fn publish(author: String, name: String, upload: String) -> Json<Value> {
    println!("{:?}/{:?}: {:?}", author, name, upload);
    Json(json!({ "dashboard": {}}))
}

#[get("/login/<session>")]
fn login(session: String) -> ApiResult<Redirect> {
    let client = Authenticator::from_env()?;
    let (url, _csrf) = client.request_auth(session);
    Ok(Redirect::to(&url.to_string()))
}

#[get("/whoami")]
fn whoami(session: AuthHeader, connection: db::Connection) -> ApiResult<Json<ApiResponse<WhoamiResponse>>> {
    let user = session.verify(&connection)?;
    Ok(Json(ApiResponse::Success(WhoamiResponse {
        user,
    })))
}

#[derive(Debug, FromForm)]
pub struct OAuth {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

impl OAuth {
    pub fn extract(self) -> Result<(String, String)> {
        match (self.code, self.state, self.error, self.error_description) {
            (Some(code), Some(state), None, None) => Ok((code, state)),
            (_, _, Some(error), Some(error_description)) => bail!("oauth error: {:?}, {:?}", error, error_description),
            _ => bail!("Invalid request"),
        }
    }
}

#[get("/authorize?<auth>")]
fn authorize(auth: OAuth, connection: db::Connection) -> ApiResult<Json<Value>> {
    let (code, state) = auth.extract()?;
    let client = Authenticator::from_env()?;
    client.store_code(code, state, &connection)?;

    Ok(Json(json!({ "success": {}})))
}
