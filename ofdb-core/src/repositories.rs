// Low-level database access traits.
// Each repository is responsible for a single entity and
// its relationships. Related entities are only referenced
// by their id and never modified or loaded by another
// repository.

use crate::entities::*;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The requested object could not be found")]
    NotFound,
    #[error("The object already exists")]
    AlreadyExists,
    #[error("The version of the object is invalid")]
    InvalidVersion,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

type Result<T> = std::result::Result<T, Error>;

// TODO: Rename to CommentRepo
pub trait CommentRepository {
    fn create_comment(&self, _: Comment) -> Result<()>;

    // Only unarchived comments
    fn load_comment(&self, id: &str) -> Result<Comment>;
    fn load_comments(&self, id: &[&str]) -> Result<Vec<Comment>>;
    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>>;

    // Only unarchived comments (even if the rating has already been archived)
    fn zip_ratings_with_comments(
        &self,
        ratings: Vec<Rating>,
    ) -> Result<Vec<(Rating, Vec<Comment>)>> {
        let mut results = Vec::with_capacity(ratings.len());
        for rating in ratings {
            debug_assert!(rating.archived_at.is_none());
            let comments = self.load_comments_of_rating(rating.id.as_ref())?;
            results.push((rating, comments));
        }
        Ok(results)
    }

    fn archive_comments(&self, ids: &[&str], activity: &Activity) -> Result<usize>;
    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        activity: &Activity,
    ) -> Result<usize>;
    fn archive_comments_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize>;
}

// TODO: Rename to RatingRepo
pub trait RatingRepository {
    fn create_rating(&self, rating: Rating) -> Result<()>;

    // Only unarchived ratings without comments
    fn load_rating(&self, id: &str) -> Result<Rating>;
    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>>;
    fn load_ratings_of_place(&self, place_id: &str) -> Result<Vec<Rating>>;

    fn archive_ratings(&self, ids: &[&str], activity: &Activity) -> Result<usize>;
    fn archive_ratings_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize>;

    fn load_place_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>>;
}

pub trait UserTokenRepo {
    fn replace_user_token(&self, user_token: UserToken) -> Result<EmailNonce>;

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken>;

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize>;

    fn get_user_token_by_email(&self, email: &EmailAddress) -> Result<UserToken>;
}

pub trait ReviewTokenRepo {
    fn add_review_token(&self, review_token: &ReviewToken) -> Result<()>;
    fn consume_review_token(&self, review_nonce: &ReviewNonce) -> Result<ReviewToken>;
    fn delete_expired_review_tokens(&self, expired_before: Timestamp) -> Result<usize>;
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, Hash)]
pub struct Pagination {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct RecentlyChangedEntriesParams {
    pub since: Option<Timestamp>,
    pub until: Option<Timestamp>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MostPopularTagsParams {
    pub min_count: Option<u64>,
    pub max_count: Option<u64>,
}

pub trait PlaceRepo {
    // TODO: use typed ID
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)>;
    // TODO: use typed IDs
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>>;
    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>>; // TODO: remove
    fn count_places(&self) -> Result<usize>;

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>>;

    fn find_places_not_updated_since(
        &self,
        not_updated_since: Timestamp,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus)>>;

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>>;

    // TODO: use typed IDs
    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize>;

    fn create_or_update_place(&self, place: Place) -> Result<()>;

    // TODO: use typed ID
    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory>;

    // TODO: use typed ID
    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)>;
}

pub trait OrganizationRepo {
    fn create_org(&mut self, org: Organization) -> Result<()>;
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization>;
    fn map_tag_to_clearance_org_id(&self, tag: &str) -> Result<Option<Id>>;
    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> Result<Vec<(Id, ModeratedTag)>>;
}

pub trait PlaceClearanceRepo {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize>;
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64>;
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>>;
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>>;
    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize>;
    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64>;
}

pub trait EventRepo {
    fn create_event(&self, event: Event) -> Result<()>;
    fn update_event(&self, event: &Event) -> Result<()>;
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize>;

    fn get_event(&self, id: &str) -> Result<Event>;
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>>;

    fn all_events_chronologically(&self) -> Result<Vec<Event>>;

    fn count_events(&self) -> Result<usize>;

    // Delete an event, but only if tagged with at least one of the given tags.
    // If no tags are provided the event is deleted unconditionally.
    // Ok(true)  => Found and deleted
    // Ok(false) => Found but no matching tags
    // TODO: Use explicit result semantics
    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool>;

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool>;
}

pub trait UserRepo {
    fn create_user(&self, user: &User) -> Result<()>;
    fn update_user(&self, user: &User) -> Result<()>;
    fn delete_user_by_email(&self, email: &EmailAddress) -> Result<()>;

    fn all_users(&self) -> Result<Vec<User>>;
    fn count_users(&self) -> Result<usize>;

    fn get_user_by_email(&self, email: &EmailAddress) -> Result<User>;
    fn try_get_user_by_email(&self, email: &EmailAddress) -> Result<Option<User>>;
}

pub trait SubscriptionRepo {
    fn create_bbox_subscription(&self, bbox: &BboxSubscription) -> Result<()>;
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>>;
    fn all_bbox_subscriptions_by_email(
        &self,
        user_email: &EmailAddress,
    ) -> Result<Vec<BboxSubscription>>;
    fn delete_bbox_subscriptions_by_email(&self, user_email: &EmailAddress) -> Result<()>;
    // TODO: add
    // fn bbox_subscriptions_affected_by_place(&self, place: &Place) -> Result<()>;
}

pub trait ReminderRepo {
    fn find_last_sent_reminder(
        &self,
        place_id: &Id,
        email: &EmailAddress,
    ) -> Result<Option<Timestamp>>;

    // TODO: use this fn:
    // fn find_last_sent_reminders(
    //     &self,
    //     place_ids: &[&Id],
    // ) -> Result<HashMap<(Id, EmailAddress), Timestamp>>;

    fn save_sent_reminders(
        &self,
        place_id: &Id,
        recipients: &[EmailAddress],
        sent_at: Timestamp,
    ) -> Result<()>;
}

pub trait TagRepo {
    fn create_tag_if_it_does_not_exist(&self, tag: &Tag) -> Result<()>;
    fn all_tags(&self) -> Result<Vec<Tag>>;
    fn count_tags(&self) -> Result<usize>;
}

// TODO: remove
pub trait CategoryRepo {
    fn all_categories(&self) -> Result<Vec<Category>> {
        Ok(vec![
            Category::new_non_profit(),
            Category::new_commercial(),
            Category::new_event(),
        ])
    }
}
