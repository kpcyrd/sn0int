use errors::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use schema::*;


#[derive(AsChangeset, Serialize, Deserialize, Queryable, Insertable)]
#[table_name="auth_tokens"]
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

#[derive(AsChangeset, Identifiable, Queryable, Serialize, PartialEq, Debug)]
#[table_name="modules"]
pub struct Module {
    pub id: i32,
    pub author: String,
    pub name: String,
    pub description: String,
    pub latest: Option<String>,
}

impl Module {
    pub fn create(module: &NewModule, connection: &PgConnection) -> Result<Module> {
        diesel::insert_into(modules::table)
            .values(module)
            .get_result(connection)
            .map_err(Error::from)
    }

    pub fn find(author: &str, name: &str, connection: &PgConnection) -> Result<Module> {
        modules::table.filter(modules::columns::author.eq(author))
                        .filter(modules::columns::name.eq(name))
                        .first::<Self>(connection)
                        .map_err(Error::from)
    }

    pub fn find_opt(author: &str, name: &str, connection: &PgConnection) -> Result<Option<Module>> {
        modules::table.filter(modules::columns::author.eq(author))
                        .filter(modules::columns::name.eq(name))
                        .first::<Self>(connection)
                        .optional()
                        .map_err(Error::from)
    }

    pub fn update_or_create(author: &str, name: &str, description: &str, connection: &PgConnection) -> Result<Module> {
        match Self::find_opt(author, name, connection)? {
            Some(module) => diesel::update(modules::table.filter(modules::columns::id.eq(module.id)))
                            .set(modules::columns::description.eq(description))
                            .get_result(connection)
                            .map_err(Error::from),
            None => Self::create(&NewModule {
                author,
                name,
                description,
                latest: None,
            }, connection),
        }
    }

    pub fn id(id: i32, connection: &PgConnection) -> Result<Module> {
        modules::table.find(id)
            .first::<Module>(connection)
            .map_err(Error::from)
    }

    pub fn id_opt(id: i32, connection: &PgConnection) -> Result<Option<Module>> {
        modules::table.find(id)
            .first::<Module>(connection)
            .optional()
            .map_err(Error::from)
    }

    pub fn delete(id: i32, connection: &PgConnection) -> Result<()> {
        diesel::delete(modules::table.find(id))
            .execute(connection)?;
        Ok(())
    }

    pub fn add_version(&self, version: &str, code: &str, connection: &PgConnection) -> Result<()> {
        let _release = Release::create(&NewRelease {
            module_id: self.id,
            version,
            code,
        }, connection)?;

        diesel::update(modules::table.filter(modules::columns::id.eq(self.id)))
            .set(modules::columns::latest.eq(version))
            .execute(connection)?;

        Ok(())
    }
}

#[derive(Insertable)]
#[table_name="modules"]
pub struct NewModule<'a> {
    author: &'a str,
    name: &'a str,
    description: &'a str,
    latest: Option<&'a str>,
}

#[derive(AsChangeset, Identifiable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[belongs_to(Module)]
#[table_name="releases"]
pub struct Release {
    pub id: i32,
    pub module_id: i32,
    pub version: String,
    pub downloads: i32,
    pub code: String,
}

impl Release {
    pub fn create(release: &NewRelease, connection: &PgConnection) -> Result<Release> {
        diesel::insert_into(releases::table)
            .values(release)
            .get_result(connection)
            .map_err(Error::from)
        /*
        releases::table.filter(releases::columns::module_id.eq(release.module_id))
                        .filter(releases::columns::version.eq(&release.version))
                        .select(releases::columns::id)
                        .first::<i32>(connection)
                        .map_err(Error::from)
        */
    }

    pub fn find(id: i32, version: &str, connection: &PgConnection) -> Result<Release> {
        releases::table.filter(releases::columns::id.eq(id))
                        .filter(releases::columns::version.eq(version))
                        .first::<Release>(connection)
                        .map_err(Error::from)
    }

    pub fn id(id: i32, connection: &PgConnection) -> Result<Release> {
        releases::table.find(id)
            .first::<Release>(connection)
            .map_err(Error::from)
    }

    pub fn id_opt(id: i32, connection: &PgConnection) -> Result<Option<Release>> {
        releases::table.find(id)
            .first::<Release>(connection)
            .optional()
            .map_err(Error::from)
    }

    pub fn delete(id: i32, connection: &PgConnection) -> Result<()> {
        diesel::delete(releases::table.find(id))
            .execute(connection)?;
        Ok(())
    }
}

#[derive(Insertable)]
#[table_name="releases"]
pub struct NewRelease<'a> {
    module_id: i32,
    version: &'a str,
    code: &'a str,
}
