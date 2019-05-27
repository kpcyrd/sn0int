use crate::assets::{ASSET_REV, FAVICON, STYLE_SHEET};
use crate::db;
use crate::errors::ApiResult;
use crate::models::*;
use rocket::http::{ContentType, Status};
use rocket::http::hyper::header::{CacheControl, CacheDirective};
use rocket_contrib::templates::Template;


#[get("/")]
pub fn index(connection: db::Connection) -> ApiResult<Template> {
    let modules = Module::start_page(&connection)?;
    let asset_rev = ASSET_REV.as_str();
    Ok(Template::render("index", json!({
        "ASSET_REV": asset_rev,
        "modules": modules,
    })))
}

#[derive(Responder)]
pub struct CachableResponder {
    inner: Vec<u8>,
    content_type: ContentType,
    cache: CacheControl,
}

impl CachableResponder {
    pub fn new<I: Into<Vec<u8>>>(inner: I, content_type: ContentType, max_age: u32) -> CachableResponder {
        let cache = CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(max_age),
        ]);
        CachableResponder {
            inner: inner.into(),
            content_type,
            cache,
        }
    }

    pub fn immutable<I: Into<Vec<u8>>>(inner: I, content_type: ContentType) -> CachableResponder {
        let cache = CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(31536000),
            CacheDirective::Extension("immutable".into(), None),
        ]);
        CachableResponder {
            inner: inner.into(),
            content_type,
            cache,
        }
    }
}

#[get("/favicon.ico")]
pub fn favicon() -> CachableResponder {
    CachableResponder::new(FAVICON, ContentType::Binary, 3600)
}

#[get("/assets/<rev>/style.css")]
pub fn style(rev: String) -> Result<CachableResponder, Status> {
    if rev == *ASSET_REV {
        Ok(CachableResponder::immutable(STYLE_SHEET, ContentType::CSS))
    } else {
        Err(Status::NotFound)
    }
}
