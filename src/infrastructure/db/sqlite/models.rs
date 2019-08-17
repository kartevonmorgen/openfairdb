use super::schema::*;

use crate::core::{
    entities,
    util::{nonce::Nonce, rowid::RowId},
};

#[derive(Queryable, Insertable)]
#[table_name = "entries"]
pub struct Entry {
    pub id: String,
    pub osm_node: Option<i64>,
    pub created: i64,
    pub archived: Option<i64>,
    pub version: i64,
    pub current: bool,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lng: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "events"]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub start: i64,
    pub end: Option<i64>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub homepage: Option<String>,
    pub created_by: Option<String>,
    pub registration: Option<i16>,
    pub organizer: Option<String>,
    pub archived: Option<i64>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable, Insertable)]
#[table_name = "organizations"]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub api_token: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "categories"]
pub struct Category {
    pub id: String,
    pub created: i64,
    pub version: i64,
    pub name: String,
}

#[derive(Queryable)]
pub struct EntryCategoryRelation {
    pub entry_id: String,
    pub entry_version: i64,
    pub category_id: String,
}

#[derive(Insertable)]
#[table_name = "entry_category_relations"]
pub struct StoreableEntryCategoryRelation<'a, 'b> {
    pub entry_id: &'a str,
    pub entry_version: i64,
    pub category_id: &'b str,
}

#[derive(Queryable)]
pub struct EntryTagRelation {
    pub entry_id: String,
    pub entry_version: i64,
    pub tag_id: String,
}

#[derive(Insertable)]
#[table_name = "entry_tag_relations"]
pub struct StoreableEntryTagRelation<'a, 'b> {
    pub entry_id: &'a str,
    pub entry_version: i64,
    pub tag_id: &'b str,
}

#[derive(Queryable)]
pub struct EventTagRelation {
    pub event_id: String,
    pub tag_id: String,
}

#[derive(Insertable)]
#[table_name = "event_tag_relations"]
pub struct StoreableEventTagRelation<'a, 'b> {
    pub event_id: &'a str,
    pub tag_id: &'b str,
}

#[derive(Queryable)]
pub struct OrgTagRelation {
    pub org_id: String,
    pub tag_id: String,
}

#[derive(Insertable)]
#[table_name = "org_tag_relations"]
pub struct StoreableOrgTagRelation<'a, 'b> {
    pub org_id: &'a str,
    pub tag_id: &'b str,
}

#[derive(Queryable, Insertable)]
#[table_name = "tags"]
pub struct Tag {
    pub id: String,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[table_name = "users"]
pub struct User {
    pub id: String, // TOTO: remove
    pub username: String,
    pub password: String,
    pub email: String,
    pub email_confirmed: bool,
    pub role: i16,
}

#[derive(Queryable, Insertable)]
#[table_name = "comments"]
pub struct Comment {
    pub id: String,
    pub created: i64,
    pub archived: Option<i64>,
    pub text: String,
    pub rating_id: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "ratings"]
pub struct Rating {
    pub id: String,
    pub created: i64,
    pub archived: Option<i64>,
    pub title: String,
    pub value: i32,
    pub context: String,
    pub source: Option<String>,
    pub entry_id: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "bbox_subscriptions"]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
    pub username: String,
}

#[derive(Queryable)]
pub struct EmailTokenCredentials {
    pub id: i64,
    pub expires_at: i64,
    pub username: String,
    pub email: String,
    pub nonce: String,
}

impl From<EmailTokenCredentials> for entities::EmailTokenCredentials {
    fn from(from: EmailTokenCredentials) -> Self {
        Self {
            expires_at: from.expires_at.into(),
            username: from.username,
            token: entities::EmailToken {
                email: from.email,
                nonce: from.nonce.parse::<Nonce>().unwrap_or_default(),
            },
        }
    }
}

impl From<EmailTokenCredentials> for (RowId, entities::EmailTokenCredentials) {
    fn from(from: EmailTokenCredentials) -> Self {
        let rowid = from.id.into();
        let entity = from.into();
        (rowid, entity)
    }
}

#[derive(Insertable, AsChangeset)]
#[table_name = "email_token_credentials"]
pub struct NewEmailTokenCredentials<'a, 'b> {
    pub expires_at: i64,
    pub username: &'a str,
    pub email: &'b str,
    pub nonce: String,
}

impl<'a, 'b, 'x> From<&'x entities::EmailTokenCredentials> for NewEmailTokenCredentials<'a, 'b>
where
    'x: 'a + 'b,
{
    fn from(from: &'x entities::EmailTokenCredentials) -> Self {
        Self {
            expires_at: from.expires_at.into(),
            username: &from.username,
            email: &from.token.email,
            nonce: from.token.nonce.to_string(),
        }
    }
}
