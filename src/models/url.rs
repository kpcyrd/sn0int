use errors::*;
use json::LuaJsonValue;
use models::*;
use ser;
use serde_json;


#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(Subdomain)]
#[table_name="urls"]
pub struct Url {
    pub id: i32,
    pub subdomain_id: i32,
    pub value: String,
    pub status: Option<i32>,
    pub body: Option<Vec<u8>>,
}

#[derive(Insertable)]
#[table_name="urls"]
pub struct NewUrl<'a> {
    pub subdomain_id: i32,
    pub value: &'a str,
    pub status: Option<i32>,
    pub body: Option<&'a [u8]>,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="urls"]
pub struct NewUrlOwned {
    pub subdomain_id: i32,
    pub value: String,
    pub status: Option<i32>,
    #[serde(deserialize_with="ser::opt_string_or_bytes")]
    pub body: Option<Vec<u8>>,
}

impl NewUrlOwned {
    pub fn from_lua(x: LuaJsonValue) -> Result<NewUrlOwned> {
        let x = serde_json::from_value(x.into())?;
        Ok(x)
    }
}
