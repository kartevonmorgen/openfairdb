use super::*;

impl<'a> PlaceRepo for DbReadOnly<'a> {
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)> {
        self.inner().get_place(id)
    }
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>> {
        self.inner().get_places(ids)
    }

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>> {
        self.inner().all_places()
    }
    fn count_places(&self) -> Result<usize> {
        self.inner().count_places()
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
        self.inner().recently_changed_places(params, pagination)
    }

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        self.inner()
            .most_popular_place_revision_tags(params, pagination)
    }

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize> {
        self.inner().review_places(ids, status, activity)
    }

    fn create_or_update_place(&self, place: Place) -> Result<()> {
        self.inner().create_or_update_place(place)
    }

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory> {
        self.inner().get_place_history(id, revision)
    }

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)> {
        self.inner().load_place_revision(id, rev)
    }
}

impl<'a> PlaceClearanceRepo for DbReadOnly<'a> {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize> {
        self.inner()
            .add_pending_clearance_for_places(org_ids, pending_clearance)
    }
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        self.inner().count_pending_clearances_for_places(org_id)
    }
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        self.inner()
            .list_pending_clearances_for_places(org_id, pagination)
    }
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>> {
        self.inner()
            .load_pending_clearances_for_places(org_id, place_ids)
    }
    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize> {
        self.inner()
            .update_pending_clearances_for_places(org_id, clearances)
    }
    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        self.inner().cleanup_pending_clearances_for_places(org_id)
    }
}

impl<'a> OrganizationRepo for DbReadOnly<'a> {
    fn create_org(&mut self, org: Organization) -> Result<()> {
        self.inner().create_org(org)
    }
    fn get_org_by_api_token(&self, token: &str) -> Result<Organization> {
        self.inner().get_org_by_api_token(token)
    }
    fn map_tag_to_clearance_org_id(&self, tag: &str) -> Result<Option<Id>> {
        self.inner().map_tag_to_clearance_org_id(tag)
    }
    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> Result<Vec<(Id, ModeratedTag)>> {
        self.inner().get_moderated_tags_by_org(excluded_org_id)
    }
}

impl<'a> CommentRepository for DbReadOnly<'a> {
    fn create_comment(&self, comment: Comment) -> Result<()> {
        self.inner().create_comment(comment)
    }
    fn load_comment(&self, id: &str) -> Result<Comment> {
        self.inner().load_comment(id)
    }
    fn load_comments(&self, id: &[&str]) -> Result<Vec<Comment>> {
        self.inner().load_comments(id)
    }
    fn load_comments_of_rating(&self, rating_id: &str) -> Result<Vec<Comment>> {
        self.inner().load_comments_of_rating(rating_id)
    }

    fn archive_comments(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        self.inner().archive_comments(ids, activity)
    }
    fn archive_comments_of_ratings(
        &self,
        rating_ids: &[&str],
        activity: &Activity,
    ) -> Result<usize> {
        self.inner()
            .archive_comments_of_ratings(rating_ids, activity)
    }
    fn archive_comments_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        self.inner().archive_comments_of_places(place_ids, activity)
    }
}

impl<'a> RatingRepository for DbReadOnly<'a> {
    fn create_rating(&self, rating: Rating) -> Result<()> {
        self.inner().create_rating(rating)
    }

    fn load_rating(&self, id: &str) -> Result<Rating> {
        self.inner().load_rating(id)
    }
    fn load_ratings(&self, ids: &[&str]) -> Result<Vec<Rating>> {
        self.inner().load_ratings(ids)
    }
    fn load_ratings_of_place(&self, place_id: &str) -> Result<Vec<Rating>> {
        self.inner().load_ratings_of_place(place_id)
    }

    fn archive_ratings(&self, ids: &[&str], activity: &Activity) -> Result<usize> {
        self.inner().archive_ratings(ids, activity)
    }
    fn archive_ratings_of_places(&self, place_ids: &[&str], activity: &Activity) -> Result<usize> {
        self.inner().archive_ratings_of_places(place_ids, activity)
    }

    fn load_place_ids_of_ratings(&self, ids: &[&str]) -> Result<Vec<String>> {
        self.inner().load_place_ids_of_ratings(ids)
    }
}

impl<'a> UserTokenRepo for DbReadOnly<'a> {
    fn replace_user_token(&self, user_token: UserToken) -> Result<EmailNonce> {
        self.inner().replace_user_token(user_token)
    }

    fn consume_user_token(&self, email_nonce: &EmailNonce) -> Result<UserToken> {
        self.inner().consume_user_token(email_nonce)
    }

    fn delete_expired_user_tokens(&self, expired_before: Timestamp) -> Result<usize> {
        self.inner().delete_expired_user_tokens(expired_before)
    }

    fn get_user_token_by_email(&self, email: &str) -> Result<UserToken> {
        self.inner().get_user_token_by_email(email)
    }
}

impl<'a> EventRepo for DbReadOnly<'a> {
    fn create_event(&self, ev: Event) -> Result<()> {
        self.inner().create_event(ev)
    }
    fn update_event(&self, ev: &Event) -> Result<()> {
        self.inner().update_event(ev)
    }
    fn archive_events(&self, ids: &[&str], archived: Timestamp) -> Result<usize> {
        self.inner().archive_events(ids, archived)
    }

    fn get_event(&self, id: &str) -> Result<Event> {
        self.inner().get_event(id)
    }
    fn get_events_chronologically(&self, ids: &[&str]) -> Result<Vec<Event>> {
        self.inner().get_events_chronologically(ids)
    }

    fn all_events_chronologically(&self) -> Result<Vec<Event>> {
        self.inner().all_events_chronologically()
    }

    fn count_events(&self) -> Result<usize> {
        self.inner().count_events()
    }

    fn delete_event_with_matching_tags(&self, id: &str, tags: &[&str]) -> Result<bool> {
        self.inner().delete_event_with_matching_tags(id, tags)
    }

    fn is_event_owned_by_any_organization(&self, id: &str) -> Result<bool> {
        self.inner().is_event_owned_by_any_organization(id)
    }
}

impl<'a> UserRepo for DbReadOnly<'a> {
    fn create_user(&self, user: &User) -> Result<()> {
        self.inner().create_user(user)
    }
    fn update_user(&self, user: &User) -> Result<()> {
        self.inner().update_user(user)
    }
    fn delete_user_by_email(&self, email: &str) -> Result<()> {
        self.inner().delete_user_by_email(email)
    }

    fn all_users(&self) -> Result<Vec<User>> {
        self.inner().all_users()
    }
    fn count_users(&self) -> Result<usize> {
        self.inner().count_users()
    }

    fn get_user_by_email(&self, email: &str) -> Result<User> {
        self.inner().get_user_by_email(email)
    }
    fn try_get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        self.inner().try_get_user_by_email(email)
    }
}

impl<'a> Db for DbReadOnly<'a> {
    fn create_tag_if_it_does_not_exist(&self, tag: &Tag) -> Result<()> {
        self.inner().create_tag_if_it_does_not_exist(tag)
    }
    fn all_tags(&self) -> Result<Vec<Tag>> {
        self.inner().all_tags()
    }
    fn count_tags(&self) -> Result<usize> {
        self.inner().count_tags()
    }

    fn create_bbox_subscription(&self, sub: &BboxSubscription) -> Result<()> {
        self.inner().create_bbox_subscription(sub)
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        self.inner().all_bbox_subscriptions()
    }
    fn all_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<Vec<BboxSubscription>> {
        self.inner().all_bbox_subscriptions_by_email(user_email)
    }
    fn delete_bbox_subscriptions_by_email(&self, user_email: &str) -> Result<()> {
        self.inner().delete_bbox_subscriptions_by_email(user_email)
    }
}
