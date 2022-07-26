use crate::{entities::*, repositories::*};
use anyhow::Result as Fallible;

type Result<T> = std::result::Result<T, Error>;

//TODO:
//  - TagGeatway
//  - SubscriptionGateway

pub trait Db:
    PlaceRepo
    + UserRepo
    + EventRepo
    + OrganizationRepo
    + CommentRepository
    + RatingRepository
    + UserTokenRepo
    + PlaceClearanceRepo
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

#[derive(Copy, Clone, Debug)]
pub enum IndexQueryMode {
    WithRating,
    WithoutRating,
}

#[derive(Debug, Default, Clone)]
pub struct IndexQuery<'a, 'b> {
    // status = None: Don't filter by review status, i.e. return all entries
    //          independent of their current review status
    // status = Some(empty vector): Exclude all invisible/inexistent entries, i.e.
    //          return only visible/existent entries
    // status = Some(non-empty vector): Include entries only if their current review
    //          status matches one of the given values
    pub status: Option<Vec<ReviewStatus>>,
    pub include_bbox: Option<MapBbox>,
    pub exclude_bbox: Option<MapBbox>,
    pub categories: Vec<&'a str>,
    pub ids: Vec<&'b str>,
    pub hash_tags: Vec<String>,
    pub text_tags: Vec<String>,
    pub text: Option<String>,
    pub ts_min_lb: Option<Timestamp>, // lower bound (inclusive)
    pub ts_min_ub: Option<Timestamp>, // upper bound (inclusive)
    pub ts_max_lb: Option<Timestamp>, // lower bound (inclusive)
    pub ts_max_ub: Option<Timestamp>, // upper bound (inclusive)
}

pub trait Indexer {
    fn flush_index(&mut self) -> Fallible<()>;
}

pub trait IdIndex {
    fn query_ids(
        &self,
        mode: IndexQueryMode,
        query: &IndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Id>>;
}

pub trait IdIndexer: Indexer + IdIndex {
    fn remove_by_id(&self, id: &Id) -> Fallible<()>;
}

#[derive(Debug, Default, Clone)]
pub struct IndexedPlace {
    pub id: String,
    pub status: Option<ReviewStatus>,
    pub pos: MapPoint,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub ratings: AvgRatings,
}

pub trait PlaceIndex {
    fn query_places(&self, query: &IndexQuery, limit: usize) -> Fallible<Vec<IndexedPlace>>;
}

pub trait PlaceIndexer: IdIndexer + PlaceIndex {
    fn add_or_update_place(
        &self,
        place: &Place,
        status: ReviewStatus,
        ratings: &AvgRatings,
    ) -> Fallible<()>;
}

pub trait EventIndexer: IdIndexer {
    fn add_or_update_event(&self, event: &Event) -> Fallible<()>;
}

pub trait EventAndPlaceIndexer: PlaceIndexer + EventIndexer {}
