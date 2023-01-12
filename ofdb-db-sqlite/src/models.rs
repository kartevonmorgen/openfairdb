#![allow(clippy::extra_unused_lifetimes)]

// NOTE:
// All timestamps with the `_at` postfix are stored
// as unix timestamp in **milli**seconds.
//
// TODO: Create a new type for milliseconds and seconds.

use super::schema::*;

#[derive(Insertable)]
#[diesel(table_name = place)]
pub struct NewPlace<'a, 'b> {
    pub id: &'a str,
    pub license: &'b str,
    pub current_rev: i64,
}

#[derive(Queryable)]
pub struct Place {
    pub rowid: i64,
    pub current_rev: i64,
    pub id: String,
    pub license: String,
}

#[derive(Insertable)]
#[diesel(table_name = place_revision)]
pub struct NewPlaceRevision {
    pub parent_rowid: i64,
    pub rev: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub current_status: i16,
    pub title: String,
    pub description: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub opening_hours: Option<String>,
    pub founded_on: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
}

#[derive(Queryable)]
pub struct JoinedPlaceRevision {
    pub id: i64,
    pub rev: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub current_status: i16,
    pub title: String,
    pub desc: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub opening_hours: Option<String>,
    pub founded_on: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    // Joined columns
    pub place_id: String,
    pub place_license: String,
}

#[derive(Queryable)]
pub struct JoinedPlaceRevisionWithStatusReview {
    pub id: i64,
    pub rev: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub title: String,
    pub desc: String,
    pub lat: f64,
    pub lon: f64,
    pub street: Option<String>,
    pub zip: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub homepage: Option<String>,
    pub opening_hours: Option<String>,
    pub founded_on: Option<String>,
    pub image_url: Option<String>,
    pub image_link_url: Option<String>,
    // Joined columns
    pub place_id: String,
    pub place_license: String,
    pub review_rev: i64,
    pub review_created_at: i64,
    pub review_created_by: Option<i64>,
    pub review_status: i16,
    pub review_context: Option<String>,
    pub review_comment: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = place_revision_review)]
pub struct NewPlaceReviewedRevision<'a, 'b> {
    pub parent_rowid: i64,
    pub rev: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub status: i16,
    pub context: Option<&'a str>,
    pub comment: Option<&'b str>,
}

#[derive(Queryable)]
pub struct PlaceReviewedRevision {
    pub rev: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub created_by_email: Option<String>,
    pub status: i16,
    pub context: Option<String>,
    pub comment: Option<String>,
}

#[derive(Queryable)]
pub struct PlaceRevisionTag {
    pub parent_rowid: i64,
    pub tag: String,
}

#[derive(Insertable)]
#[diesel(table_name = place_revision_tag)]
pub struct NewPlaceRevisionTag<'a> {
    pub parent_rowid: i64,
    pub tag: &'a str,
}

#[derive(Queryable)]
pub struct PlaceRevisionCustomLink {
    pub parent_rowid: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = place_revision_custom_link)]
pub struct NewPlaceRevisionCustomLink<'a> {
    pub parent_rowid: i64,
    pub url: &'a str,
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
}

#[derive(Insertable)]
#[diesel(table_name = place_rating)]
pub struct NewPlaceRating {
    pub parent_rowid: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub id: String,
    pub title: String,
    pub value: i16,
    pub context: String,
    pub source: Option<String>,
}

#[derive(Queryable)]
pub struct PlaceRating {
    pub rowid: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub id: String,
    pub title: String,
    pub value: i16,
    pub context: String,
    pub source: Option<String>,
    // Joined columns
    pub place_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = place_rating_comment)]
pub struct NewPlaceRatingComment {
    pub parent_rowid: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub id: String,
    pub text: String,
}

#[derive(Queryable)]
pub struct PlaceRatingComment {
    pub rowid: i64,
    pub created_at: i64,
    pub created_by: Option<i64>,
    pub archived_at: Option<i64>,
    pub archived_by: Option<i64>,
    pub id: String,
    pub text: String,

    pub rating_id: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = events)]
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
    pub state: Option<String>,
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
    pub state: Option<String>,
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

#[derive(Insertable)]
#[diesel(table_name = organization)]
pub struct NewOrganization {
    pub id: String,
    pub name: String,
    pub api_token: String,
}

#[derive(Queryable)]
pub struct Organization {
    pub rowid: i64,
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
#[diesel(table_name = event_tags)]
pub struct NewEventTag<'a> {
    pub event_id: i64,
    pub tag: &'a str,
}

#[derive(Queryable)]
pub struct OrganizationTag {
    pub org_rowid: i64,
    pub tag_label: String,
    pub tag_allow_add: i16,
    pub tag_allow_remove: i16,
    pub require_clearance: i16,
}

#[derive(Queryable)]
pub struct OrganizationTagWithId {
    pub org_id: String,
    pub tag_label: String,
    pub tag_allow_add: i16,
    pub tag_allow_remove: i16,
    pub require_clearance: i16,
}

#[derive(Insertable)]
#[diesel(table_name = organization_tag)]
pub struct NewOrganizationTag<'a> {
    pub org_rowid: i64,
    pub tag_label: &'a str,
    pub tag_allow_add: i16,
    pub tag_allow_remove: i16,
    pub require_clearance: i16,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = users)]
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
#[diesel(table_name = bbox_subscriptions)]
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
#[diesel(table_name = user_tokens)]
pub struct NewUserToken {
    pub user_id: i64,
    pub nonce: String,
    pub expires_at: i64,
}

#[derive(Queryable)]
pub struct UserTokenEntity {
    pub user_id: i64,
    pub nonce: String,
    pub expires_at: i64,
    // Joined columns
    pub user_email: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = review_tokens)]
pub struct NewReviewToken {
    pub place_rowid: i64,
    pub revision: i64,
    pub expires_at: i64,
    pub nonce: String,
}

#[derive(Queryable)]
pub struct ReviewTokenEntity {
    pub place_id: String,
    pub place_revision: i64,
    pub expires_at: i64,
    pub nonce: String,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = organization_place_clearance)]
#[diesel(treat_none_as_null = true)]
pub struct NewPendingClearanceForPlace {
    pub org_rowid: i64,
    pub place_rowid: i64,
    pub created_at: i64,
    pub last_cleared_revision: Option<i64>,
}

#[derive(Queryable)]
pub struct PendingClearanceForPlace {
    pub place_id: String,
    pub created_at: i64,
    pub last_cleared_revision: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = sent_reminders)]
pub struct NewSentReminder<'a> {
    pub place_rowid: i64,
    pub sent_at: i64,
    pub sent_to_email: &'a str,
}

#[derive(Queryable)]
pub struct SentReminder {
    pub place_id: String,
    pub sent_at: i64,
    pub sent_to_email: String,
}
