use rocket::http::Status;
use rocket::response::content;
use rocket_contrib::Template;
use assets::{ASSET_REV, FAVICON, STYLE_SHEET};


#[get("/")]
fn index() -> Template {
    Template::render("index", hashmap!{
        "ASSET_REV" => ASSET_REV.as_str(),
    })
}

#[get("/favicon.ico")]
fn favicon() -> Vec<u8> {
    FAVICON.to_vec()
}

#[get("/assets/<rev>/style.css")]
fn style(rev: String) -> Result<content::Css<&'static str>, Status> {
    if rev == *ASSET_REV {
        Ok(content::Css(STYLE_SHEET))
    } else {
        Err(Status::NotFound)
    }
}
