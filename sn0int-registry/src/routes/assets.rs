use crate::assets::*;
use rocket::http::{ContentType, Status};
use rocket::http::hyper::header::{CacheControl, CacheDirective};


#[derive(Responder)]
pub struct CachableResponder {
    inner: &'static [u8],
    content_type: ContentType,
    cache: CacheControl,
}

impl CachableResponder {
    pub fn new(inner: &'static [u8], content_type: ContentType, max_age: u32) -> CachableResponder {
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

    pub fn immutable(inner: &'static [u8], content_type: ContentType) -> CachableResponder {
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
        Ok(CachableResponder::immutable(STYLESHEET, ContentType::CSS))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/assets/<rev>/script.js")]
pub fn javascript(rev: String) -> Result<CachableResponder, Status> {
    if rev == *ASSET_REV {
        Ok(CachableResponder::immutable(JAVASCRIPT, ContentType::JavaScript))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/assets/social-card.png")]
pub fn social_card() -> CachableResponder {
    CachableResponder::new(SOCIAL_CARD, ContentType::PNG, 3600)
}

#[get("/assets/<rev>/clipboard.min.js")]
pub fn javascript_clipboard(rev: String) -> Result<CachableResponder, Status> {
    if rev == *ASSET_REV {
        Ok(CachableResponder::immutable(JAVASCRIPT_CLIPBOARD, ContentType::JavaScript))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/assets/<rev>/fontawesome/css/all.min.css")]
pub fn fontawesome_style(rev: String) -> Result<CachableResponder, Status> {
    if rev == *ASSET_REV {
        Ok(CachableResponder::immutable(FA_STYLESHEET, ContentType::CSS))
    } else {
        Err(Status::NotFound)
    }
}

#[get("/assets/<rev>/fontawesome/webfonts/<font>")]
pub fn fontawesome_font(rev: String, font: String) -> Result<CachableResponder, Status> {
    if rev != *ASSET_REV {
        return Err(Status::NotFound)
    }

    let (content_type, bytes) = match font.as_str() {
        "fa-brands-400.svg" => (ContentType::SVG, FA_BRANDS_400_SVG),
        "fa-brands-400.ttf" => (ContentType::TTF, FA_BRANDS_400_TTF),
        "fa-brands-400.woff" => (ContentType::WOFF, FA_BRANDS_400_WOFF),
        "fa-brands-400.woff2" => (ContentType::WOFF2, FA_BRANDS_400_WOFF2),
        "fa-regular-400.svg" => (ContentType::SVG, FA_REGULAR_400_SVG),
        "fa-regular-400.ttf" => (ContentType::TTF, FA_REGULAR_400_TTF),
        "fa-regular-400.woff" => (ContentType::WOFF, FA_REGULAR_400_WOFF),
        "fa-regular-400.woff2" => (ContentType::WOFF2, FA_REGULAR_400_WOFF2),
        "fa-solid-900.svg" => (ContentType::SVG, FA_SOLID_900_SVG),
        "fa-solid-900.ttf" => (ContentType::TTF, FA_SOLID_900_TTF),
        "fa-solid-900.woff" => (ContentType::WOFF, FA_SOLID_900_WOFF),
        "fa-solid-900.woff2" => (ContentType::WOFF2, FA_SOLID_900_WOFF2),
        _ => return Err(Status::NotFound),
    };

    Ok(CachableResponder::immutable(bytes, content_type))
}
