use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::fmt::colors::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use crate::models::*;
use std::sync::Arc;
use crate::engine::ctx::State;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="images"]
pub struct Image {
    pub id: i32,
    pub value: String,

    pub filename: Option<String>,
    pub mime: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created: Option<NaiveDateTime>,

    pub latitude: Option<f32>,
    pub longitude: Option<f32>,

    pub nudity: Option<f32>,
    pub ahash: Option<String>,
    pub dhash: Option<String>,
    pub phash: Option<String>,

    pub unscoped: bool,
}

impl Model for Image {
    type ID = str;

    fn to_string(&self) -> String {
        self.value.to_owned()
    }

    fn list(db: &Database) -> Result<Vec<Self>> {
        use crate::schema::images::dsl::*;

        let results = images.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use crate::schema::images::dsl::*;

        let query = images.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn delete(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::images::dsl::*;

        diesel::delete(images.filter(filter.sql()))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn delete_id(db: &Database, my_id: i32) -> Result<usize> {
        use crate::schema::images::dsl::*;

        diesel::delete(images.filter(id.eq(my_id)))
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
        use crate::schema::images::dsl::*;

        let domain = images.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(domain)
    }

    fn get(db: &Database, query: &Self::ID) -> Result<Self> {
        use crate::schema::images::dsl::*;

        let email = images.filter(value.eq(query))
            .first::<Self>(db.db())?;

        Ok(email)
    }

    fn get_opt(db: &Database, query: &Self::ID) -> Result<Option<Self>> {
        use crate::schema::images::dsl::*;

        let email = images.filter(value.eq(query))
            .first::<Self>(db.db())
            .optional()?;

        Ok(email)
    }

    fn blob(&self) -> Option<&str> {
        Some(&self.value)
    }
}

impl Scopable for Image {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn set_scoped(&self, db: &Database, my_value: bool) -> Result<()> {
        use crate::schema::images::dsl::*;
        diesel::update(images.filter(id.eq(self.id)))
            .set(unscoped.eq(!my_value))
            .execute(db.db())?;
        Ok(())
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::images::dsl::*;

        diesel::update(images.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use crate::schema::images::dsl::*;

        diesel::update(images.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

pub struct PrintableImage {
    value: String,
    filename: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
}

impl fmt::Display for PrintableImage {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{:?}", self.value)?;
        if self.filename.is_none() && self.height.is_none() && self.height.is_none() {
            return Ok(());
        }

        write!(w, " (")?;
        let mut dirty = false;

        if let Some(filename) = &self.filename {
            write!(w, "{:?}", filename)?;
            dirty = true;
        }

        if let (Some(width), Some(height)) = (self.width, self.height) {
            if dirty {
                write!(w, ", ")?;
            }
            write!(w, "{}x{}", width, height)?;
        }

        write!(w, " )")
    }
}

impl Printable<PrintableImage> for Image {
    fn printable(&self, _db: &Database) -> Result<PrintableImage> {
        Ok(PrintableImage {
            value: self.value.to_string(),
            filename: self.filename.clone(),
            width: self.width,
            height: self.height,
        })
    }
}

pub struct DetailedImage {
    id: i32,
    value: String,

    filename: Option<String>,
    mime: Option<String>,
    width: Option<i32>,
    height: Option<i32>,
    created: Option<NaiveDateTime>,

    latitude: Option<f32>,
    longitude: Option<f32>,

    nudity: Option<f32>,
    ahash: Option<String>,
    dhash: Option<String>,
    phash: Option<String>,

    unscoped: bool,
}

impl DisplayableDetailed for DetailedImage {
    #[inline]
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    #[inline]
    fn print(&self, w: &mut fmt::DetailFormatter) -> fmt::Result {
        w.id(self.id)?;
        w.debug::<Green, _>(&self.value)?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.filename)?;
        w.opt_debug::<Yellow, _>(&self.mime)?;

        if let (Some(width), Some(height)) = (self.width, self.height) {
            w.display::<Yellow, _>(&format!("{}x{}", width, height))?;
        }

        w.opt_debug::<Yellow, _>(&self.created)?;
        w.end_group()?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.latitude)?;
        w.opt_debug::<Yellow, _>(&self.longitude)?;
        w.end_group()?;

        w.start_group();
        w.opt_debug::<Yellow, _>(&self.nudity)?;
        w.opt_debug::<Yellow, _>(&self.ahash)?;
        w.opt_debug::<Yellow, _>(&self.dhash)?;
        w.opt_debug::<Yellow, _>(&self.phash)?;
        w.end_group()?;

        Ok(())
    }

    #[inline]
    fn children(&self, _w: &mut fmt::DetailFormatter) -> fmt::Result {
        Ok(())
    }
}

display_detailed!(DetailedImage);

impl Detailed for Image {
    type T = DetailedImage;

    fn detailed(&self, _db: &Database) -> Result<Self::T> {
        Ok(DetailedImage {
            id: self.id,
            value: self.value.to_string(),

            filename: self.filename.clone(),
            mime: self.mime.clone(),
            width: self.width,
            height: self.height,
            created: self.created,

            latitude: self.latitude,
            longitude: self.longitude,

            nudity: self.nudity,
            ahash: self.ahash.clone(),
            dhash: self.dhash.clone(),
            phash: self.phash.clone(),

            unscoped: self.unscoped,
        })
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="images"]
pub struct NewImage {
    pub value: String,

    pub filename: Option<String>,
    pub mime: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created: Option<NaiveDateTime>,

    pub latitude: Option<f32>,
    pub longitude: Option<f32>,

    pub nudity: Option<f32>,
    pub ahash: Option<String>,
    pub dhash: Option<String>,
    pub phash: Option<String>,

    pub unscoped: bool,
}

impl InsertableStruct<Image> for NewImage {
    fn value(&self) -> &str {
        &self.value
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.unscoped = !scoped;
    }

    fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(images::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

impl Upsertable<Image> for NewImage {
    type Update = ImageUpdate;

    fn upsert(self, existing: &Image) -> Self::Update {
        Self::Update {
            id: existing.id,

            filename: Self::upsert_opt(self.filename, &existing.filename),
            mime: Self::upsert_opt(self.mime, &existing.mime),
            width: Self::upsert_opt(self.width, &existing.width),
            height: Self::upsert_opt(self.height, &existing.height),
            created: Self::upsert_opt(self.created, &existing.created),

            latitude: Self::upsert_opt(self.latitude, &existing.latitude),
            longitude: Self::upsert_opt(self.longitude, &existing.longitude),

            nudity: Self::upsert_opt(self.nudity, &existing.nudity),
            ahash: Self::upsert_opt(self.ahash, &existing.ahash),
            dhash: Self::upsert_opt(self.dhash, &existing.dhash),
            phash: Self::upsert_opt(self.phash, &existing.phash),
        }
    }
}

impl Printable<PrintableImage> for NewImage {
    fn printable(&self, _db: &Database) -> Result<PrintableImage> {
        Ok(PrintableImage {
            value: self.value.to_string(),
            filename: self.filename.clone(),
            width: self.width,
            height: self.height,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertImage {
    pub value: String,

    pub filename: Option<String>,
    pub mime: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created: Option<NaiveDateTime>,

    pub latitude: Option<f32>,
    pub longitude: Option<f32>,

    pub nudity: Option<f32>,
    pub ahash: Option<String>,
    pub dhash: Option<String>,
    pub phash: Option<String>,
}

impl LuaInsertToNew for InsertImage {
    type Target = NewImage;

    fn lua_try_into_new(self, state: &Arc<dyn State>) -> Result<NewImage> {
        // TODO: enforce this rule for updates as well
        if let Some(filename) = &self.filename {
            if filename.contains('/') {
                bail!("filename can't contains slashes"); // TODO: automatically extract filename
            }
        }

        state.persist_blob(&self.value)?;

        Ok(NewImage {
            value: self.value,

            filename: self.filename,
            mime: self.mime,
            width: self.width,
            height: self.height,
            created: self.created,

            latitude: self.latitude,
            longitude: self.longitude,

            nudity: self.nudity,
            ahash: self.ahash,
            dhash: self.dhash,
            phash: self.phash,

            unscoped: false,
        })
    }
}

#[derive(Identifiable, AsChangeset, Serialize, Deserialize, Debug)]
#[table_name="images"]
pub struct ImageUpdate {
    pub id: i32,

    pub filename: Option<String>,
    pub mime: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created: Option<NaiveDateTime>,

    pub latitude: Option<f32>,
    pub longitude: Option<f32>,

    pub nudity: Option<f32>,
    pub ahash: Option<String>,
    pub dhash: Option<String>,
    pub phash: Option<String>,
}

impl Upsert for ImageUpdate {
    fn is_dirty(&self) -> bool {
        self.filename.is_some() ||
        self.mime.is_some() ||
        self.width.is_some() ||
        self.height.is_some() ||
        self.created.is_some() ||

        self.latitude.is_some() ||
        self.longitude.is_some() ||

        self.nudity.is_some() ||
        self.ahash.is_some() ||
        self.dhash.is_some() ||
        self.phash.is_some()
    }

    fn generic(self) -> Update {
        Update::Image(self)
    }

    fn apply(&self, db: &Database) -> Result<i32> {
        db.update_image(self)
    }
}

impl Updateable<Image> for ImageUpdate {
    fn changeset(&mut self, existing: &Image) {
        Self::clear_if_equal(&mut self.filename, &existing.filename);
        Self::clear_if_equal(&mut self.mime, &existing.mime);
        Self::clear_if_equal(&mut self.width, &existing.width);
        Self::clear_if_equal(&mut self.height, &existing.height);
        Self::clear_if_equal(&mut self.created, &existing.created);

        Self::clear_if_equal(&mut self.latitude, &existing.latitude);
        Self::clear_if_equal(&mut self.longitude, &existing.longitude);

        Self::clear_if_equal(&mut self.nudity, &existing.nudity);
        Self::clear_if_equal(&mut self.ahash, &existing.ahash);
        Self::clear_if_equal(&mut self.dhash, &existing.dhash);
        Self::clear_if_equal(&mut self.phash, &existing.phash);
    }

    fn fmt(&self, updates: &mut Vec<String>, colors: bool) {
        Self::push_value(updates, "filename", &self.filename, colors);
        Self::push_value(updates, "mime", &self.mime, colors);
        Self::push_value(updates, "width", &self.width, colors);
        Self::push_value(updates, "height", &self.height, colors);
        Self::push_value(updates, "created", &self.created, colors);

        Self::push_value(updates, "latitude", &self.latitude, colors);
        Self::push_value(updates, "longitude", &self.longitude, colors);

        Self::push_value(updates, "nudity", &self.nudity, colors);
        Self::push_value(updates, "ahash", &self.ahash, colors);
        Self::push_value(updates, "dhash", &self.dhash, colors);
        Self::push_value(updates, "phash", &self.phash, colors);
    }
}
