use crate::errors::*;
use crate::fmt::Write;
use crate::fmt::colors::*;
use crate::models::*;
use diesel;
use diesel::prelude::*;
use crate::ser;
use crate::url;
use std::sync::Arc;
use crate::engine::ctx::State;


#[derive(Identifiable, Queryable, Associations, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Subdomain)]
#[table_name="urls"]
pub struct Url {
    pub id: i32,
    pub subdomain_id: i32,
    pub value: String,
    pub path: String,
    pub status: Option<i32>,
    pub body: Option<Vec<u8>>,
    pub unscoped: bool,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
}

impl Model for Url {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::urls::dsl::*;

        let results = urls.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::urls::dsl::*;

        let query = urls.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::urls::dsl::*;

        diesel::delete(urls.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::urls::dsl::*;

        diesel::delete(urls.filter(id.eq(my_id)))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn id(&self) -> i32 {
        self.id
    }

    fn value(&self) -> &Self::ID {
        &self.value
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use crate::schema::urls::dsl::*;

        let url = urls.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::urls::dsl::*;

        let url = urls.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(url)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::urls::dsl::*;

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
        use crate::schema::urls::dsl::*;

        diesel::update(urls.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::urls::dsl::*;

        diesel::update(urls.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
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

impl DisplayableDetailed for DetailedUrl {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        if let Some(status) = self.status {
            write!(w, " (")?;
            w.display::<Green, _>(status)?;

            if let Some(ref redirect) = self.redirect {
                write!(w, " => ")?;
                w.debug::<Green, _>(redirect)?;
            }

            write!(w, ")")?;
        }

        if let Some(ref title) = self.title {
            write!(w, " {:?}", title)?;
        }

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedUrl);

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

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="urls"]
pub struct NewUrl {
    pub subdomain_id: i32,
    pub value: String,
    pub path: String,
    pub status: Option<i32>,
    #[serde(deserialize_with="ser::opt_string_or_bytes")]
    pub body: Option<Vec<u8>>,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
    pub unscoped: bool,
}

impl InsertableStruct<Url> for NewUrl {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(urls::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Url> for NewUrl {
    type Update = UrlUpdate;

    fn upsert(self, existing: &Url) -> Self::Update {
        Self::Update {
            id: existing.id,
            status: Self::upsert_opt(self.status, &existing.status),
            body: Self::upsert_opt(self.body, &existing.body),
            online: Self::upsert_opt(self.online, &existing.online),
            title: Self::upsert_opt(self.title, &existing.title),
            redirect: Self::upsert_opt(self.redirect, &existing.redirect),
        }
    }
}

impl Printable<PrintableUrl> for NewUrl {
    fn printable(&self, _db: &Database) -> Result<PrintableUrl> {
        Ok(PrintableUrl {
            value: self.value.to_string(),
            status: self.status.map(|x| x as u16),
            redirect: self.redirect.clone(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertUrl {
    pub subdomain_id: i32,
    pub value: String,
    pub status: Option<i32>,
    #[serde(deserialize_with="ser::opt_string_or_bytes")]
    pub body: Option<Vec<u8>>,
    pub online: Option<bool>,
    pub title: Option<String>,
    pub redirect: Option<String>,
}

impl LuaInsertToNew for InsertUrl {
    type Target = NewUrl;

    fn try_into_new(self, _state: &Arc<State>) -> Result<NewUrl> {
        let url = url::Url::parse(&self.value)?;
        let path = url.path().to_string();

        Ok(NewUrl {
            subdomain_id: self.subdomain_id,
            value: self.value,
            path,
            status: self.status,
            body: self.body,
            online: self.online,
            title: self.title,
            redirect: self.redirect,
            unscoped: false,
        })
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

    fn generic(self) -> Update {
        Update::Url(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_url(&self)
    }
}

impl Updateable<Url> for UrlUpdate {
    fn changeset(&mut self, existing: &Url) {
        Self::clear_if_equal(&mut self.online, &existing.online);
        Self::clear_if_equal(&mut self.status, &existing.status);
        Self::clear_if_equal(&mut self.body, &existing.body);
        Self::clear_if_equal(&mut self.title, &existing.title);
        Self::clear_if_equal(&mut self.redirect, &existing.redirect);
    }

    fn fmt(&self, updates: &mut Vec<String>) {
        Self::push_value(updates, "online", &self.online);
        Self::push_value(updates, "status", &self.status);
        Self::push_raw(updates, "body", self.body.as_ref().map(|x| format!("[{} bytes]", x.len())));
        Self::push_value(updates, "title", &self.title);
        Self::push_value(updates, "redirect", &self.redirect);
    }
}
