use serde::Serialize;
use sn0int_registry::errors::*;
use sn0int_registry::db;
use sn0int_registry::models::*;
use rocket_contrib::json::Json;


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    schema_version: u8,
    label: String,
    message: String,
    color: &'static str,
}

impl Badge {
    pub fn new(label: String, message: String) -> Badge {
        Badge {
            schema_version: 1,
            label,
            message,
            color: "blue",
        }
    }
}

#[get("/modules")]
pub fn modules(connection: db::Connection) -> ApiResult<Json<Badge>> {
    let num = Module::count(&connection)?;
    Ok(Json(Badge::new("modules".into(), num.to_string())))
}

#[get("/downloads")]
pub fn downloads(connection: db::Connection) -> ApiResult<Json<Badge>> {
    let mut num = Release::downloads(&connection)?;

    let mut unit = String::new();
    while num > 1000 {
        num /= 1000;
        unit.push('k');
    }

    let message = format!("{:.1}{}", num, unit);
    Ok(Json(Badge::new("downloads".into(), message)))
}
