use super::{
    entities::*,
    error::RepoError,
    repositories::*,
    util::{
        geo::{MapBbox, MapPoint},
        time::Timestamp,
    },
};

use failure::Fallible;

type Result<T> = std::result::Result<T, RepoError>;

#[derive(Clone, Debug)]
pub struct MostPopularTagsParams {
    pub min_count: Option<u64>,
    pub max_count: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct RecentlyChangedEntriesParams {
    pub since: Option<Timestamp>,
    pub until: Option<Timestamp>,
}

pub trait EntryGateway {
    fn get_entry(&self, _: &str) -> Result<Entry>;
    fn get_entries(&self, ids: &[&str]) -> Result<Vec<Entry>>;

    fn all_entries(&self) -> Result<Vec<Entry>>;
    fn count_entries(&self) -> Result<usize>;

    fn create_entry(&self, _: Entry) -> Result<()>;
    fn update_entry(&self, _: &Entry) -> Result<()>;
    fn import_multiple_entries(&mut self, _: &[Entry]) -> Result<()>;
    fn archive_entries(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;

    fn recently_changed_entries(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<Entry>>;

    fn most_popular_entry_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>>;
}

pub trait EventGateway {
    fn create_event(&self, _: Event) -> Result<()>;
    fn get_event(&self, _: &str) -> Result<Event>;
    fn all_events(&self) -> Result<Vec<Event>>;
    fn get_events(
        &self,
        start_min: Option<Timestamp>,
        start_max: Option<Timestamp>,
    ) -> Result<Vec<Event>>;
    fn update_event(&self, _: &Event) -> Result<()>;
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;
    fn delete_event(&self, _: &str) -> Result<()>;
    fn count_events(&self) -> Result<usize>;
}

pub trait UserGateway {
    fn create_user(&self, user: User) -> Result<()>;
    fn update_user(&self, user: &User) -> Result<()>;
    fn get_user(&self, username: &str) -> Result<User>;
    //TODO make email => user relation unique
    fn get_users_by_email(&self, email: &str) -> Result<Vec<User>>;
    fn all_users(&self) -> Result<Vec<User>>;
    fn delete_user(&self, username: &str) -> Result<()>;
    fn count_users(&self) -> Result<usize>;
}

pub trait OrganizationGateway {
    fn create_org(&mut self, _: Organization) -> Result<()>;
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization>;
    fn get_all_tags_owned_by_orgs(&self) -> Result<Vec<String>>;
}

//TODO:
//  - TagGeatway
//  - CategoryGateway
//  - SubscriptionGateway

#[derive(Clone, Debug, Default)]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

pub trait Db:
    EntryGateway
    + UserGateway
    + EventGateway
    + OrganizationGateway
    + CommentRepository
    + RatingRepository
    + EmailTokenCredentialsRepository
{
    fn create_tag_if_it_does_not_exist(&self, _: &Tag) -> Result<()>;
    fn create_category_if_it_does_not_exist(&mut self, _: &Category) -> Result<()>;
    fn create_bbox_subscription(&mut self, _: &BboxSubscription) -> Result<()>;

    fn all_categories(&self) -> Result<Vec<Category>>;
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn count_tags(&self) -> Result<usize>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;

    fn delete_bbox_subscription(&mut self, _: &str) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct IndexedEntry {
    pub id: String,
    pub pos: MapPoint,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub ratings: AvgRatings,
}

#[derive(Debug, Default, Clone)]
pub struct EntryIndexQuery<'a, 'b> {
    pub include_bbox: Option<MapBbox>,
    pub exclude_bbox: Option<MapBbox>,
    pub categories: Vec<&'a str>,
    pub ids: Vec<&'b str>,
    pub hash_tags: Vec<String>,
    pub text_tags: Vec<String>,
    pub text: Option<String>,
}

pub trait EntryIndex {
    fn query_entries(&self, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<IndexedEntry>>;
}

pub trait EntryIndexer: EntryIndex {
    fn add_or_update_entry(&mut self, entry: &Entry, ratings: &AvgRatings) -> Fallible<()>;
    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()>;
    fn flush(&mut self) -> Fallible<()>;
}
