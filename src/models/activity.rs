use crate::errors::*;
use serde::{Serialize, Deserialize};
use crate::schema::activity;
use diesel::prelude::*;
use diesel::query_builder::BoxedSelectStatement;
use crate::models::*;
use chrono::NaiveDateTime;
use std::convert::TryFrom;
use std::io::Write;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name="activity"]
pub struct Activity {
    pub id: i32,
    pub topic: String,
    pub time: NaiveDateTime,
    pub uniq: Option<String>,
    pub latitude: Option<f32>,
    pub longitude: Option<f32>,
    pub radius: Option<i32>,
    pub content: String,
}

impl Activity {
    pub fn uniq(db: &Database, my_uniq: &str) -> Result<Option<Self>> {
        use crate::schema::activity::dsl::*;
        activity.filter(uniq.eq(my_uniq))
            .first::<Self>(db.db())
            .optional()
            .map_err(Error::from)
    }

    fn build_query_except_since(filter: &ActivityFilter) -> BoxedSelectStatement<(diesel::sql_types::Integer, diesel::sql_types::Text, diesel::sql_types::Timestamp, diesel::sql_types::Nullable<diesel::sql_types::Text>, diesel::sql_types::Nullable<diesel::sql_types::Float>, diesel::sql_types::Nullable<diesel::sql_types::Float>, diesel::sql_types::Nullable<diesel::sql_types::Integer>, diesel::sql_types::Text), activity::table, diesel::sqlite::Sqlite> {
        use crate::schema::activity::dsl::*;

        let mut query = activity.into_boxed();

        if let Some(my_topic) = &filter.topic {
            debug!("Filtering topic to be {:?}", my_topic);
            query = query.filter(topic.eq(my_topic));
        }

        // "since" filter is not applied

        if let Some(until) = &filter.until {
            debug!("Filtering until <= {}", until);
            query = query.filter(time.le(until));
        }

        if filter.location {
            debug!("Filtering latitude and longitude != null");
            query = query
                .filter(latitude.is_not_null())
                .filter(longitude.is_not_null());
        }

        query
    }

    pub fn query(db: &Database, filter: &ActivityFilter) -> Result<Vec<Self>> {
        use crate::schema::activity::dsl::*;

        let mut query = Self::build_query_except_since(filter);

        if let Some(since) = &filter.since {
            debug!("Filtering since >= {}", since);
            query = query.filter(time.ge(since));
        }

        query
            .order_by((time.asc(), id.asc()))
            .load::<Self>(db.db())
            .map_err(Error::from)
    }

    pub fn previous(db: &Database, previous: &Activity, filter: &ActivityFilter) -> Result<Option<Self>> {
        use crate::schema::activity::dsl::*;

        Self::build_query_except_since(filter)
            .filter(time.lt(previous.time))
            .order_by((time.desc(), id.desc()))
            .first::<Self>(db.db())
            .optional()
            .map_err(Error::from)
    }

    pub fn count(db: &Database) -> Result<usize> {
        use crate::schema::activity::dsl::*;
        activity.count()
            .get_result::<i64>(db.db())
            .map(|x| x as usize)
            .map_err(Error::from)
    }
}

pub struct ActivityFilter {
    pub topic: Option<String>,
    pub since: Option<NaiveDateTime>,
    pub until: Option<NaiveDateTime>,
    pub location: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonActivity {
    pub id: i32,
    pub initial: bool,
    pub topic: String,
    pub time: NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<i32>,
    pub content: serde_json::Value,
}

impl JsonActivity {
    pub fn write_to<W: Write>(self, mut w: W) -> Result<()> {
        let s = serde_json::to_string(&self)?;
        writeln!(w, "{}", s)?;
        Ok(())
    }
}

impl TryFrom<Activity> for JsonActivity {
    type Error = Error;

    fn try_from(a: Activity) -> Result<Self> {
        let content = serde_json::from_str(&a.content)?;

        Ok(JsonActivity {
            id: a.id,
            initial: false,
            topic: a.topic,
            time: a.time,
            uniq: a.uniq,
            latitude: a.latitude,
            longitude: a.longitude,
            radius: a.radius,
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
    pub radius: Option<i32>,
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
    pub radius: Option<i32>,
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
            radius: self.radius,
            content,
        })
    }
}
