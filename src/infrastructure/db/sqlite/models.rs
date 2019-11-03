use super::schema::*;

#[derive(Insertable)]
#[table_name = "place"]
pub struct NewPlace<'a, 'b> {
    pub uid: &'a str,
    pub lic: &'b str,
    pub rev: i64,
}

#[derive(Queryable)]
pub struct Place {
    pub id: i64,
    pub uid: String,
    pub lic: String,
    pub rev: i64,
}

#[derive(Insertable)]
#[table_name = "place_rev"]
pub struct NewPlaceRev {
    pub rev: i64,
    pub place_id: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub status: i16,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable)]
pub struct PlaceRev {
    pub id: i64,
    pub rev: i64,
    pub place_id: i64,
    pub place_uid: String,
    pub place_lic: String,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub status: i16,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable)]
pub struct PlaceRevStatusLog {
    pub id: i64,
    pub rev: i64,
    pub place_id: i64,
    pub place_uid: String,
    pub place_lic: String,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub status: i16,
    pub status_created_at: i64,
    pub status_created_by: Option<i64>,
    pub status_context: Option<String>,
    pub status_notes: Option<String>,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Insertable)]
#[table_name = "place_rev_status_log"]
pub struct NewPlaceRevStatusLog<'a, 'b> {
    pub place_rev_id: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub status: i16,
    pub context: Option<&'a str>,
    pub notes: Option<&'b str>,
}

#[derive(Queryable)]
pub struct PlaceRevTag {
    pub place_rev_id: i64,
    pub tag: String,
}

#[derive(Insertable)]
#[table_name = "place_rev_tag"]
pub struct NewPlaceRevTag<'a> {
    pub place_rev_id: i64,
    pub tag: &'a str,
}

#[derive(Insertable)]
#[table_name = "place_rating"]
pub struct NewPlaceRating {
    pub uid: String,
    pub place_id: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub title: String,
    pub value: i16,
    pub context: String,
    pub source: Option<String>,
}

#[derive(Queryable)]
pub struct PlaceRating {
    pub id: i64,
    pub uid: String,
    pub place_id: i64,
    pub place_uid: String,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub title: String,
    pub value: i16,
    pub context: String,
    pub source: Option<String>,
}

#[derive(Insertable)]
#[table_name = "place_rating_comment"]
pub struct NewPlaceRatingComment {
    pub uid: String,
    pub rating_id: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub text: String,
}

#[derive(Queryable)]
pub struct PlaceRatingComment {
    pub id: i64,
    pub uid: String,
    pub rating_id: i64,
    pub rating_uid: String,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub text: String,
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
