use super::*;

impl SubscriptionRepo for DbReadWrite<'_> {
    fn create_bbox_subscription(&self, sub: &BboxSubscription) -> Result<()> {
        create_bbox_subscription(&mut self.conn.borrow_mut(), sub)
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions(&mut self.conn.borrow_mut())
    }
    fn all_bbox_subscriptions_by_email(
        &self,
        user_email: &EmailAddress,
    ) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions_by_email(&mut self.conn.borrow_mut(), user_email)
    }
    fn delete_bbox_subscriptions_by_email(&self, user_email: &EmailAddress) -> Result<()> {
        delete_bbox_subscriptions_by_email(&mut self.conn.borrow_mut(), user_email)
    }
}

impl SubscriptionRepo for DbConnection<'_> {
    fn create_bbox_subscription(&self, sub: &BboxSubscription) -> Result<()> {
        create_bbox_subscription(&mut self.conn.borrow_mut(), sub)
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions(&mut self.conn.borrow_mut())
    }
    fn all_bbox_subscriptions_by_email(
        &self,
        user_email: &EmailAddress,
    ) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions_by_email(&mut self.conn.borrow_mut(), user_email)
    }
    fn delete_bbox_subscriptions_by_email(&self, user_email: &EmailAddress) -> Result<()> {
        delete_bbox_subscriptions_by_email(&mut self.conn.borrow_mut(), user_email)
    }
}

impl SubscriptionRepo for DbReadOnly<'_> {
    fn create_bbox_subscription(&self, _sub: &BboxSubscription) -> Result<()> {
        unreachable!();
    }
    fn all_bbox_subscriptions(&self) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions(&mut self.conn.borrow_mut())
    }
    fn all_bbox_subscriptions_by_email(
        &self,
        user_email: &EmailAddress,
    ) -> Result<Vec<BboxSubscription>> {
        all_bbox_subscriptions_by_email(&mut self.conn.borrow_mut(), user_email)
    }
    fn delete_bbox_subscriptions_by_email(&self, _user_email: &EmailAddress) -> Result<()> {
        unreachable!();
    }
}

fn create_bbox_subscription(conn: &mut SqliteConnection, new: &BboxSubscription) -> Result<()> {
    let user_id = resolve_user_created_by_email(conn, &new.user_email)?;
    let (south_west_lat, south_west_lng) = new.bbox.southwest().to_lat_lng_deg();
    let (north_east_lat, north_east_lng) = new.bbox.northeast().to_lat_lng_deg();
    let insertable = models::NewBboxSubscription {
        uid: new.id.as_ref(),
        user_id,
        south_west_lat,
        south_west_lng,
        north_east_lat,
        north_east_lng,
    };
    diesel::insert_into(schema::bbox_subscriptions::table)
        .values(&insertable)
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(())
}

fn all_bbox_subscriptions(conn: &mut SqliteConnection) -> Result<Vec<BboxSubscription>> {
    use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
    Ok(s_dsl::bbox_subscriptions
        .inner_join(u_dsl::users)
        .select((
            s_dsl::id,
            s_dsl::uid,
            s_dsl::user_id,
            s_dsl::south_west_lat,
            s_dsl::south_west_lng,
            s_dsl::north_east_lat,
            s_dsl::north_east_lng,
            u_dsl::email,
        ))
        .load::<models::BboxSubscriptionEntity>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(BboxSubscription::from)
        .collect())
}

fn all_bbox_subscriptions_by_email(
    conn: &mut SqliteConnection,
    email: &EmailAddress,
) -> Result<Vec<BboxSubscription>> {
    use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
    Ok(s_dsl::bbox_subscriptions
        .inner_join(u_dsl::users)
        .filter(u_dsl::email.eq(email.as_str()))
        .select((
            s_dsl::id,
            s_dsl::uid,
            s_dsl::user_id,
            s_dsl::south_west_lat,
            s_dsl::south_west_lng,
            s_dsl::north_east_lat,
            s_dsl::north_east_lng,
            u_dsl::email,
        ))
        .load::<models::BboxSubscriptionEntity>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(BboxSubscription::from)
        .collect())
}

fn delete_bbox_subscriptions_by_email(
    conn: &mut SqliteConnection,
    email: &EmailAddress,
) -> Result<()> {
    use schema::{bbox_subscriptions::dsl as s_dsl, users::dsl as u_dsl};
    let users_id = u_dsl::users
        .select(u_dsl::id)
        .filter(u_dsl::email.eq(email.as_str()));
    diesel::delete(s_dsl::bbox_subscriptions.filter(s_dsl::user_id.eq_any(users_id)))
        .execute(conn)
        .map_err(from_diesel_err)?;
    Ok(())
}
