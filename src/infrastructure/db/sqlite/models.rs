use super::schema::*;

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

#[derive(Insertable, AsChangeset)]
#[table_name = "events"]
pub struct NewEvent {
    pub uid: String,
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
    pub created_by: Option<i64>,
    pub registration: Option<i16>,
    pub organizer: Option<String>,
    pub archived: Option<i64>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable)]
pub struct EventEntity {
    // Table columns
    pub id: i64,
    pub uid: String,
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
    pub created_by_id: Option<i64>,
    pub registration: Option<i16>,
    pub organizer: Option<String>,
    pub archived: Option<i64>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    // Joined columns
    pub created_by_email: Option<String>,
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
pub struct EventTag {
    pub event_id: i64,
    pub tag: String,
}

#[derive(Insertable)]
#[table_name = "event_tags"]
pub struct NewEventTag<'a> {
    pub event_id: i64,
    pub tag: &'a str,
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

#[derive(Insertable, AsChangeset)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub email_confirmed: bool,
    pub password: String,
    pub role: i16,
}

#[derive(Queryable)]
pub struct UserEntity {
    pub id: i64,
    pub email: String,
    pub email_confirmed: bool,
    pub password: String,
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

#[derive(Insertable)]
#[table_name = "bbox_subscriptions"]
pub struct NewBboxSubscription<'a> {
    pub uid: &'a str,
    pub user_id: i64,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
}

#[derive(Queryable)]
pub struct BboxSubscriptionEntity {
    // Table columns
    pub id: i64,
    pub uid: String,
    pub user_id: i64,
    pub south_west_lat: f64,
    pub south_west_lng: f64,
    pub north_east_lat: f64,
    pub north_east_lng: f64,
    // Joined columns
    pub user_email: String,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "user_tokens"]
pub struct NewUserToken {
    pub user_id: i64,
    pub nonce: String,
    pub expires_at: i64,
}

#[derive(Queryable)]
pub struct UserTokenEntity {
    // Table columns
    pub user_id: i64,
    pub nonce: String,
    pub expires_at: i64,
    // Joined columns
    pub user_email: String,
}
