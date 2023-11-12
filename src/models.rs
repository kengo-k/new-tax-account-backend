use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: Option<i32>,
    pub title: String,
    pub body: String,
    pub category_id: Option<i32>,
    pub author: Option<String>,
    pub published: bool,
    pub good_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
