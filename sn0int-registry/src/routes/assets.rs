use rocket::http::Status;
use rocket::response::content;
use rocket_contrib::templates::Template;
use crate::assets::{ASSET_REV, FAVICON, STYLE_SHEET};


#[get("/")]
pub fn index() -> Template {
    Template::render("index", hashmap!{
        "ASSET_REV" => ASSET_REV.as_str(),
    })
}

#[get("/favicon.ico")]
pub fn favicon() -> Vec<u8> {
    FAVICON.to_vec()
}

#[get("/assets/<rev>/style.css")]
pub fn style(rev: String) -> Result<content::Css<&'static str>, Status> {
    if rev == *ASSET_REV {
        Ok(content::Css(STYLE_SHEET))
    } else {
        Err(Status::NotFound)
    }
}
