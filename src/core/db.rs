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

pub trait PlaceRepo {
    fn get_place(&self, uid: &str) -> Result<(Place, ReviewStatus)>;
    fn get_places(&self, uids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>>;

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>>;
    fn count_places(&self) -> Result<usize>;

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>>;

    fn most_popular_place_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>>;

    fn review_places(
        &self,
        uids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize>;

    fn create_place_rev(&self, place: Place) -> Result<()>;
}

pub trait EventGateway {
    fn create_event(&self, _: Event) -> Result<()>;
    fn update_event(&self, _: &Event) -> Result<()>;
    fn archive_events(&self, uids: &[&str], archived: Timestamp) -> Result<usize>;

    fn get_event(&self, uid: &str) -> Result<Event>;

    fn all_events(&self) -> Result<Vec<Event>>;
    fn count_events(&self) -> Result<usize>;

    fn get_events(
        &self,
        start_min: Option<Timestamp>,
        start_max: Option<Timestamp>,
    ) -> Result<Vec<Event>>;

    // Delete an event, but only if tagged with at least one of the given tags
    // Ok(Some(())) => Found and deleted
    // Ok(None)     => No matching tags
    // TODO: Use explicit result semantics
    fn delete_event_with_matching_tags(&self, uid: &str, tags: &[&str]) -> Result<Option<()>>;
}

pub trait UserGateway {
    fn create_user(&self, user: &User) -> Result<()>;
    fn update_user(&self, user: &User) -> Result<()>;
    fn delete_user_by_email(&self, email: &str) -> Result<()>;

    fn all_users(&self) -> Result<Vec<User>>;
    fn count_users(&self) -> Result<usize>;

    fn get_user_by_email(&self, email: &str) -> Result<User>;
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>>;
}

pub trait OrganizationGateway {
    fn create_org(&mut self, _: Organization) -> Result<()>;
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization>;
    fn get_all_tags_owned_by_orgs(&self) -> Result<Vec<String>>;
}

//TODO:
//  - TagGeatway
//  - SubscriptionGateway

#[derive(Clone, Debug, Default)]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

pub trait Db:
    PlaceRepo
    + UserGateway
    + EventGateway
    + OrganizationGateway
    + CommentRepository
    + RatingRepository
    + UserTokenRepo
{
    fn create_tag_if_it_does_not_exist(&self, _: &Tag) -> Result<()>;

    fn all_categories(&self) -> Result<Vec<Category>> {
        Ok(vec![
            Category::new_non_profit(),
            Category::new_commercial(),
            Category::new_event(),
        ])
    }
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn count_tags(&self) -> Result<usize>;

    fn create_bbox_subscription(&self, _: &BboxSubscription) -> Result<()>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;
    fn all_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<Vec<BboxSubscription>>;
    fn delete_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct IndexedPlace {
    pub id: String,
    pub pos: MapPoint,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub ratings: AvgRatings,
}

#[derive(Debug, Default, Clone)]
pub struct PlaceIndexQuery<'a, 'b> {
    pub include_bbox: Option<MapBbox>,
    pub exclude_bbox: Option<MapBbox>,
    pub categories: Vec<&'a str>,
    pub ids: Vec<&'b str>,
    pub hash_tags: Vec<String>,
    pub text_tags: Vec<String>,
    pub text: Option<String>,
}

pub trait PlaceIndex {
    fn query_places(&self, query: &PlaceIndexQuery, limit: usize) -> Fallible<Vec<IndexedPlace>>;
}

pub trait PlaceIndexer: PlaceIndex {
    fn add_or_update_place(&mut self, place: &Place, ratings: &AvgRatings) -> Fallible<()>;
    fn remove_place_by_uid(&mut self, uid: &str) -> Fallible<()>;
    fn flush(&mut self) -> Fallible<()>;
}
