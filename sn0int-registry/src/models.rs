use errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use schema::*;


#[table_name="auth_tokens"]
#[derive(AsChangeset, Serialize, Deserialize, Queryable, Insertable)]
pub struct AuthToken {
    pub id: String,
    pub author: String,
    pub access_token: String,
}

impl AuthToken {
    pub fn create(auth_token: &AuthToken, connection: &PgConnection) -> Result<()> {
        diesel::insert_into(auth_tokens::table)
            .values(auth_token)
            .execute(connection)?;
        Ok(())
    }

    pub fn read(id: &str, connection: &PgConnection) -> Result<AuthToken> {
        auth_tokens::table.find(id)
            .first::<AuthToken>(connection)
            .map_err(Error::from)
    }

    pub fn read_opt(id: &str, connection: &PgConnection) -> Result<Option<AuthToken>> {
        auth_tokens::table.find(id)
            .first::<AuthToken>(connection)
            .optional()
            .map_err(Error::from)
    }

    pub fn delete(id: &str, connection: &PgConnection) -> Result<()> {
        diesel::delete(auth_tokens::table.find(id))
            .execute(connection)?;
        Ok(())
    }
}
