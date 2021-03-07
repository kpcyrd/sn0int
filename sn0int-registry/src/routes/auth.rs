use crate::assets::ASSET_REV;
use crate::auth::Authenticator;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use sn0int_registry::db;
use sn0int_registry::errors::*;
use rocket::request::Form;
use rocket::response::Redirect;
use rocket_contrib::templates::Template;
use maplit::hashmap;

#[get("/?<auth..>")]
pub fn get(auth: Form<OAuth>) -> Template {
    let auth = auth.into_inner();
    let mut auth = serde_json::to_value(&auth).expect("OAuth serialization failed");
    if let Value::Object(ref mut map) = auth {
        map.insert("ASSET_REV".to_string(), Value::String(ASSET_REV.to_string()));
    }
    Template::render("auth-confirm", auth)
}

#[post("/", data="<auth>")]
pub fn post(auth: Form<OAuth>, connection: db::Connection) -> ApiResult<Template> {
    let (code, state) = auth.into_inner().extract()?;
    let client = Authenticator::from_env()?;
    client.store_code(code, state, &connection)
        .bad_request()
        .public_context("Authentication failed")?;

    Ok(Template::render("auth-done", hashmap!{
        "ASSET_REV" => ASSET_REV.as_str(),
    }))
}

#[get("/<session>")]
pub fn login(session: String) -> ApiResult<Redirect> {
    let client = Authenticator::from_env()?;
    let (url, _csrf) = client.request_auth(session);
    Ok(Redirect::to(url.to_string()))
}

#[derive(Debug, FromForm, Serialize, Deserialize)]
pub struct OAuth {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
    error_uri: Option<String>,
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
