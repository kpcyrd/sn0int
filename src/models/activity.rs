use crate::errors::*;
use crate::schema::activity;
use diesel;
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;


#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="activity"]
pub struct Activity {
    pub id: i32,
    pub topic: String,
    pub time: NaiveDateTime,
    pub uniq: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub content: String,
}

impl Activity {
    pub fn uniq(db: &Database, my_uniq: &str) -> Result<Option<Activity>> {
        use crate::schema::activity::dsl::*;
        activity.filter(uniq.eq(my_uniq))
            .first::<Self>(db.db())
            .optional()
            .map_err(|e| Error::from(e))
    }
}

#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[table_name="activity"]
pub struct NewActivity {
    pub topic: String,
    pub time: NaiveDateTime,
    pub uniq: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub content: String,
}

impl NewActivity {
    pub fn insert(&self, db: &Database) -> Result<()> {
        diesel::insert_into(activity::table)
            .values(self)
            .execute(db.db())?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertActivity {
    pub topic: String,
    pub time: NaiveDateTime,
    pub uniq: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub content: serde_json::Value,
}

impl InsertToNew for InsertActivity {
    type Target = NewActivity;

    #[inline]
    fn try_into_new(self) -> Result<NewActivity> {
        let content = serde_json::to_string(&self.content)?;
        Ok(NewActivity {
            topic: self.topic,
            time: self.time,
            uniq: self.uniq,
            latitude: self.latitude,
            longitude: self.longitude,
            content,
        })
    }
}
