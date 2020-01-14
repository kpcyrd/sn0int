use crate::errors::*;
use crate::schema::activity;
use diesel;
use diesel::prelude::*;
use crate::models::*;
use chrono::NaiveDateTime;
use std::convert::TryFrom;


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
    pub fn uniq(db: &Database, my_uniq: &str) -> Result<Option<Self>> {
        use crate::schema::activity::dsl::*;
        activity.filter(uniq.eq(my_uniq))
            .first::<Self>(db.db())
            .optional()
            .map_err(|e| Error::from(e))
    }

    pub fn query(db: &Database, filter: &ActivityFilter) -> Result<Vec<Self>> {
        use crate::schema::activity::dsl::*;

        let mut query = activity.into_boxed();

        if let Some(my_topic) = &filter.topic {
            query = query.filter(topic.eq(my_topic));
        }

        if let Some(since) = &filter.since {
            query = query.filter(time.ge(since));
        }

        if let Some(until) = &filter.until {
            query = query.filter(time.le(until));
        }

        query
            .order_by(time.asc())
            .load::<Self>(db.db())
            .map_err(Error::from)
    }
}

pub struct ActivityFilter {
    pub topic: Option<String>,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonActivity {
    pub id: i32,
    pub topic: String,
    pub time: NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f32>,
    pub content: serde_json::Value,
}

impl TryFrom<Activity> for JsonActivity {
    type Error = Error;

    fn try_from(a: Activity) -> Result<Self> {
        let content = serde_json::from_str(&a.content)?;

        Ok(JsonActivity {
            id: a.id,
            topic: a.topic,
            time: a.time,
            uniq: a.uniq,
            latitude: a.latitude,
            longitude: a.longitude,
            content,
        })
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
