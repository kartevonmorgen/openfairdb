use super::schema::*;

#[derive(Queryable, Insertable)]
#[table_name = "entries"]
pub struct Entry {
    pub id: String,
    pub osm_node: Option<i64>,
    pub created: i64,
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

#[derive(Queryable, Insertable)]
#[table_name = "categories"]
pub struct Category {
    pub id: String,
    pub created: i64,
    pub version: i64,
    pub name: String,
}

#[derive(Identifiable, Queryable, Insertable, Associations)]
#[table_name = "entry_category_relations"]
#[primary_key(entry_id, entry_version, category_id)]
pub struct EntryCategoryRelation {
    pub entry_id: String,
    pub entry_version: i64,
    pub category_id: String,
}

#[derive(Identifiable, Queryable, Insertable, Associations)]
#[table_name = "entry_tag_relations"]
#[primary_key(entry_id, entry_version, tag_id)]
pub struct EntryTagRelation {
    pub entry_id: String,
    pub entry_version: i64,
    pub tag_id: String,
}

#[derive(Queryable, Insertable)]
#[table_name = "tags"]
pub struct Tag {
    pub id: String,
}

#[derive(Identifiable, Queryable, Insertable)]
#[table_name = "users"]
#[primary_key(username)]
pub struct User {
    pub id: String, // TOTO: remove
    pub username: String,
    pub password: String,
    pub email: String,
    pub email_confirmed: bool,
}

#[derive(Queryable, Insertable)]
#[table_name = "comments"]
pub struct Comment {
    pub id: String,
    pub created: i64,
    pub text: String,
    pub rating_id: String,
}

#[derive(Queryable, Insertable, Associations)]
#[table_name = "ratings"]
#[belongs_to(Entry, foreign_key = "entry_id")]
pub struct Rating {
    pub id: String,
    pub created: i64,
    pub title: String,
    pub value: i32,
    pub context: String,
    pub source: Option<String>,
    pub entry_id: String,
}

#[derive(Queryable, Insertable, Associations)]
#[table_name = "bbox_subscriptions"]
#[belongs_to(User, foreign_key = "username")]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
    pub username: String,
}
