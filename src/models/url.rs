use errors::*;
use diesel;
use diesel::prelude::*;
use models::*;
use ser;


#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Subdomain)]
#[table_name="urls"]
pub struct Url {
    pub id: i32,
    pub subdomain_id: i32,
    pub value: String,
    pub status: Option<i32>,
    pub body: Option<Vec<u8>>,
    pub unscoped: bool,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Model for Url {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::urls::dsl::*;

        let results = urls.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::urls::dsl::*;

        let query = urls.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::urls::dsl::*;

        diesel::delete(urls.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::urls::dsl::*;

        let url = urls.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::urls::dsl::*;

        let url_id = urls.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(url_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::urls::dsl::*;

        let url_id = urls.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(url_id)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use schema::urls::dsl::*;

        let url = urls.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use schema::urls::dsl::*;

        let url = urls.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(url)
    }
}

impl Scopable for Url {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::urls::dsl::*;

        diesel::update(urls.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::urls::dsl::*;

        diesel::update(urls.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="urls"]
pub struct UrlUpdate {
    pub id: i32,
    pub status: Option<i32>,
    pub body: Option<Vec<u8>>,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
}

impl Upsert for UrlUpdate {
    fn is_dirty(&self) -> bool {
        self.status.is_some() ||
        self.body.is_some() ||
        self.online.is_some() ||
        self.title.is_some() ||
        self.redirect.is_some()
    }
}

impl fmt::Display for UrlUpdate {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        let mut updates = Vec::new();

        if let Some(online) = self.online {
            updates.push(format!("online => {:?}", online));
        }
        if let Some(status) = self.status {
            updates.push(format!("status => {:?}", status));
        }
        if let Some(ref body) = self.body {
            updates.push(format!("body => [{} bytes]", body.len()));
        }
        if let Some(ref title) = self.title {
            updates.push(format!("title => {:?}", title));
        }
        if let Some(ref redirect) = self.redirect {
            updates.push(format!("redirect => {:?}", redirect));
        }

        write!(w, "{}", updates.join(", "))
    }
}

pub struct PrintableUrl {
    value: String,
    status: Option<u16>,
    redirect: Option<String>,
}

impl fmt::Display for PrintableUrl {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)?;

        if let Some(status) = self.status {
            write!(w, " ({}", status)?;

            if let Some(ref redirect) = self.redirect {
                write!(w, " => {:?}", redirect)?;
            }

            write!(w, ")")?;
        }

        Ok(())
    }
}

impl Printable<PrintableUrl> for Url {
    fn printable(&self, _db: &Database) -> Result<PrintableUrl> {
        Ok(PrintableUrl {
            value: self.value.to_string(),
            status: self.status.map(|x| x as u16),
            redirect: self.redirect.clone(),
        })
    }
}

pub struct DetailedUrl {
    id: i32,
    value: String,
    status: Option<u16>,
    unscoped: bool,
    title: Option<String>,
    redirect: Option<String>,
}

impl fmt::Display for DetailedUrl {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.unscoped {
            write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{:?}\x1b[0m", self.id, self.value)?;

            if let Some(status) = self.status {
                write!(w, " (\x1b[33m{}\x1b[0m", status)?;

                if let Some(ref redirect) = self.redirect {
                    write!(w, " => \x1b[33m{:?}\x1b[0m", redirect)?;
                }

                write!(w, ")")?;
            }

            if let Some(ref title) = self.title {
                write!(w, " {:?}", title)?;
            }
        } else {
            write!(w, "\x1b[90m#{}, {:?}\x1b[0m", self.id, self.value)?;

            if let Some(status) = self.status {
                write!(w, "\x1b[90m ({})\x1b[0m", status)?;
            }
        }

        Ok(())
    }
}

impl Detailed for Url {
    type T = DetailedUrl;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedUrl {
            id: self.id,
            value: self.value.to_string(),
            status: self.status.map(|x| x as u16),
            unscoped: self.unscoped,
            title: self.title.clone(),
            redirect: self.redirect.clone(),
        })
    }
}

#[derive(Insertable)]
#[table_name="urls"]
pub struct NewUrl<'a> {
    pub subdomain_id: i32,
    pub value: &'a str,
    pub status: Option<i32>,
    pub body: Option<&'a [u8]>,
    pub online: Option<bool>,
    pub title: Option<&'a str>,
    pub redirect: Option<&'a str>,
}

impl<'a> Upsertable for NewUrl<'a> {
    type Struct = Url;
    type Update = UrlUpdate;

    fn upsert(&self, existing: &Self::Struct) -> Self::Update {
        Self::Update {
            id: existing.id,
            status: if self.status != existing.status { self.status } else { None },
            body: if self.body != existing.body.as_ref().map(|x| &x[..]) { self.body.map(|x| x.to_owned()) } else { None },
            online: if self.online != existing.online { self.online } else { None },
            title: if self.title != existing.title.as_ref().map(|x| x.as_str()) { self.title.map(|x| x.to_owned()) } else { None },
            redirect: if self.redirect != existing.redirect.as_ref().map(|x| x.as_str()) { self.redirect.map(|x| x.to_owned()) } else { None },
        }
    }
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="urls"]
pub struct NewUrlOwned {
    pub subdomain_id: i32,
    pub value: String,
    pub status: Option<i32>,
    #[serde(deserialize_with="ser::opt_string_or_bytes")]
    pub body: Option<Vec<u8>>,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
}

impl Printable<PrintableUrl> for NewUrlOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableUrl> {
        Ok(PrintableUrl {
            value: self.value.to_string(),
            status: self.status.map(|x| x as u16),
            redirect: self.redirect.clone(),
        })
    }
}
