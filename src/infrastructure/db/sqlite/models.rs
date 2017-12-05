use super::schema::*;

#[derive(Queryable, Insertable)]
#[table_name = "entries"]
pub struct Entry {
    pub id: String,
    pub created: i32,
    pub version: i32,
    pub current: bool,
    pub title: String,
    pub description: String,
    pub lat: f32,
    pub lng: f32,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
}

#[derive(Queryable, Insertable)]
#[table_name = "categories"]
pub struct Category {
    pub id: String,
    pub created: i32,
    pub version: i32,
    pub name: String,
}

#[derive(Identifiable, Queryable, Insertable, Associations)]
#[table_name = "entry_category_relations"]
#[primary_key(entry_id, entry_version, category_id)]
pub struct EntryCategoryRelation {
    pub entry_id: String,
    pub entry_version: i32,
    pub category_id: String,
}

#[derive(Identifiable, Queryable, Insertable, Associations)]
#[table_name = "entry_tag_relations"]
#[primary_key(entry_id, entry_version, tag_id)]
pub struct EntryTagRelation {
    pub entry_id: String,
    pub entry_version: i32,
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
    pub username: String,
    pub password: String,
    pub email: String,
    pub email_confirmed: bool,
}

#[derive(Queryable, Insertable)]
#[table_name = "comments"]
pub struct Comment {
    pub id: String,
    pub created: i32,
    pub text: String,
    pub rating_id: Option<String>, //TODO remove option
}

#[derive(AsChangeset)]
#[table_name = "comments"]
#[changeset_options(treat_none_as_null = "true")]
pub struct CommentUpdate {
    pub rating_id: Option<String>, //TODO remove option
}

#[derive(Queryable, Insertable)]
#[table_name = "ratings"]
pub struct Rating {
    pub id: String,
    pub created: i32,
    pub title: String,
    pub value: i32,
    pub context: String,
    pub source: Option<String>,
    pub entry_id: Option<String>, //TODO remove option
}

#[derive(AsChangeset)]
#[table_name = "ratings"]
#[changeset_options(treat_none_as_null = "true")]
pub struct RatingUpdate {
    pub entry_id: Option<String>, //TODO remove option
}

#[derive(Queryable, Insertable)]
#[table_name = "bbox_subscriptions"]
pub struct BboxSubscription {
    pub id: String,
    pub south_west_lat: f32,
    pub south_west_lng: f32,
    pub north_east_lat: f32,
    pub north_east_lng: f32,
    pub user_id: Option<String>, //TODO remove option
}

#[derive(AsChangeset)]
#[table_name = "bbox_subscriptions"]
#[changeset_options(treat_none_as_null = "true")]
pub struct BboxSubscriptionUpdate {
    pub user_id: Option<String>, //TODO remove option
}
