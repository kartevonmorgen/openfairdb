use super::*;

impl<'a> PlaceRepo for DbReadWrite<'a> {
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)> {
        get_place(&mut self.conn.borrow_mut(), id)
    }
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>> {
        get_places(&mut self.conn.borrow_mut(), ids)
    }

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>> {
        all_places(&mut self.conn.borrow_mut())
    }
    fn count_places(&self) -> Result<usize> {
        count_places(&mut self.conn.borrow_mut())
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
        recently_changed_places(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        most_popular_place_revision_tags(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize> {
        review_places(&mut self.conn.borrow_mut(), ids, status, activity)
    }

    fn create_or_update_place(&self, place: Place) -> Result<()> {
        create_or_update_place(&mut self.conn.borrow_mut(), place)
    }

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory> {
        get_place_history(&mut self.conn.borrow_mut(), id, revision)
    }

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)> {
        load_place_revision(&mut self.conn.borrow_mut(), id, rev)
    }
}

impl<'a> PlaceRepo for DbConnection<'a> {
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)> {
        get_place(&mut self.conn.borrow_mut(), id)
    }
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>> {
        get_places(&mut self.conn.borrow_mut(), ids)
    }

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>> {
        all_places(&mut self.conn.borrow_mut())
    }
    fn count_places(&self) -> Result<usize> {
        count_places(&mut self.conn.borrow_mut())
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
        recently_changed_places(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        most_popular_place_revision_tags(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize> {
        review_places(&mut self.conn.borrow_mut(), ids, status, activity)
    }

    fn create_or_update_place(&self, place: Place) -> Result<()> {
        create_or_update_place(&mut self.conn.borrow_mut(), place)
    }

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory> {
        get_place_history(&mut self.conn.borrow_mut(), id, revision)
    }

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)> {
        load_place_revision(&mut self.conn.borrow_mut(), id, rev)
    }
}

impl<'a> PlaceRepo for DbReadOnly<'a> {
    fn get_place(&self, id: &str) -> Result<(Place, ReviewStatus)> {
        get_place(&mut self.conn.borrow_mut(), id)
    }
    fn get_places(&self, ids: &[&str]) -> Result<Vec<(Place, ReviewStatus)>> {
        get_places(&mut self.conn.borrow_mut(), ids)
    }

    fn all_places(&self) -> Result<Vec<(Place, ReviewStatus)>> {
        all_places(&mut self.conn.borrow_mut())
    }
    fn count_places(&self) -> Result<usize> {
        count_places(&mut self.conn.borrow_mut())
    }

    fn recently_changed_places(
        &self,
        params: &RecentlyChangedEntriesParams,
        pagination: &Pagination,
    ) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
        recently_changed_places(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn most_popular_place_revision_tags(
        &self,
        params: &MostPopularTagsParams,
        pagination: &Pagination,
    ) -> Result<Vec<TagFrequency>> {
        most_popular_place_revision_tags(&mut self.conn.borrow_mut(), params, pagination)
    }

    fn review_places(
        &self,
        ids: &[&str],
        status: ReviewStatus,
        activity: &ActivityLog,
    ) -> Result<usize> {
        review_places(&mut self.conn.borrow_mut(), ids, status, activity)
    }

    fn create_or_update_place(&self, place: Place) -> Result<()> {
        create_or_update_place(&mut self.conn.borrow_mut(), place)
    }

    fn get_place_history(&self, id: &str, revision: Option<Revision>) -> Result<PlaceHistory> {
        get_place_history(&mut self.conn.borrow_mut(), id, revision)
    }

    fn load_place_revision(&self, id: &str, rev: Revision) -> Result<(Place, ReviewStatus)> {
        load_place_revision(&mut self.conn.borrow_mut(), id, rev)
    }
}

fn create_or_update_place(conn: &mut SqliteConnection, place: Place) -> Result<()> {
    let (_place_id, new_place, tags, custom_links) = into_new_place_revision(conn, place)?;
    diesel::insert_into(schema::place_revision::table)
        .values(&new_place)
        .execute(conn)
        .map_err(from_diesel_err)?;

    use schema::place_revision::dsl;
    let parent_rowid = schema::place_revision::table
        .select(dsl::rowid)
        .filter(dsl::parent_rowid.eq(new_place.parent_rowid))
        .filter(dsl::rev.eq(new_place.rev))
        .first::<i64>(conn)
        .map_err(|e| {
            log::warn!(
                "Newly inserted place {} revision {} not found: {}",
                new_place.parent_rowid,
                new_place.rev,
                e
            );
            e
        })
        .map_err(from_diesel_err)?;

    // Insert into place_revision_review
    let new_review = models::NewPlaceReviewedRevision {
        parent_rowid,
        rev: u64::from(Revision::initial()) as i64,
        created_at: new_place.created_at,
        created_by: new_place.created_by,
        status: new_place.current_status,
        context: None,
        comment: Some("created"),
    };
    diesel::insert_into(schema::place_revision_review::table)
        .values(new_review)
        .execute(conn)
        .map_err(from_diesel_err)?;

    // Insert into place_revision_tag
    let insertable_tags: Vec<_> = tags
        .iter()
        .map(|tag| models::NewPlaceRevisionTag {
            parent_rowid,
            tag: tag.as_str(),
        })
        .collect();
    diesel::insert_into(schema::place_revision_tag::table)
        .values(&insertable_tags)
        .execute(conn)
        .map_err(from_diesel_err)?;

    // Insert into place_revision_custom_link
    let insertable_custom_links: Vec<_> = custom_links
        .iter()
        .map(
            |CustomLink {
                 url,
                 title,
                 description,
             }| models::NewPlaceRevisionCustomLink {
                parent_rowid,
                url: url.as_str(),
                title: title.as_ref().map(String::as_str),
                description: description.as_ref().map(String::as_str),
            },
        )
        .collect();
    diesel::insert_into(schema::place_revision_custom_link::table)
        .values(&insertable_custom_links)
        .execute(conn)
        .map_err(from_diesel_err)?;

    Ok(())
}

fn review_places(
    conn: &mut SqliteConnection,
    ids: &[&str],
    status: ReviewStatus,
    activity_log: &ActivityLog,
) -> Result<usize> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};

    let rev_ids = schema::place_revision::table
        .inner_join(
            schema::place::table.on(rev_dsl::parent_rowid
                .eq(dsl::rowid)
                .and(rev_dsl::rev.eq(dsl::current_rev))),
        )
        .select(rev_dsl::rowid)
        .filter(dsl::id.eq_any(ids))
        .filter(rev_dsl::current_status.ne(ReviewStatusPrimitive::from(status)))
        .load(conn)
        .map_err(from_diesel_err)?;
    let ActivityLog {
        activity,
        context,
        comment,
    } = activity_log;
    let changed_at = activity.at.as_millis();
    let changed_by = if let Some(ref email) = activity.by {
        Some(resolve_user_created_by_email(conn, email.as_ref())?)
    } else {
        None
    };
    let status = ReviewStatusPrimitive::from(status);
    let mut total_update_count = 0;
    for rev_id in rev_ids {
        let update_count = diesel::update(
            schema::place_revision::table
                .filter(rev_dsl::rowid.eq(rev_id))
                .filter(rev_dsl::current_status.ne(status)),
        )
        .set(rev_dsl::current_status.eq(status))
        .execute(conn)
        .map_err(from_diesel_err)?;
        debug_assert!(update_count <= 1);
        if update_count > 0 {
            use schema::place_revision_review::dsl as review_dsl;
            let prev_rev = Revision::from(
                schema::place_revision_review::table
                    .select(diesel::dsl::max(review_dsl::rev))
                    .filter(review_dsl::parent_rowid.eq(rev_id))
                    .first::<Option<i64>>(conn)
                    .map_err(from_diesel_err)?
                    .ok_or(repo::Error::NotFound)? as u64,
            );
            let next_rev = prev_rev.next();
            let new_review = models::NewPlaceReviewedRevision {
                parent_rowid: rev_id,
                rev: u64::from(next_rev) as i64,
                status,
                created_at: changed_at,
                created_by: changed_by,
                context: context.as_deref(),
                comment: comment.as_deref(),
            };
            diesel::insert_into(schema::place_revision_review::table)
                .values(new_review)
                .execute(conn)
                .map_err(from_diesel_err)?;
            total_update_count += update_count;
        }
    }
    Ok(total_update_count)
}

fn get_places(
    conn: &mut SqliteConnection,
    place_ids: &[&str],
) -> Result<Vec<(Place, ReviewStatus)>> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};

    let mut query = schema::place_revision::table
        .inner_join(
            schema::place::table.on(rev_dsl::parent_rowid
                .eq(dsl::rowid)
                .and(rev_dsl::rev.eq(dsl::current_rev))),
        )
        .select((
            rev_dsl::rowid,
            rev_dsl::rev,
            rev_dsl::created_at,
            rev_dsl::created_by,
            rev_dsl::current_status,
            rev_dsl::title,
            rev_dsl::description,
            rev_dsl::lat,
            rev_dsl::lon,
            rev_dsl::street,
            rev_dsl::zip,
            rev_dsl::city,
            rev_dsl::country,
            rev_dsl::state,
            rev_dsl::contact_name,
            rev_dsl::email,
            rev_dsl::phone,
            rev_dsl::homepage,
            rev_dsl::opening_hours,
            rev_dsl::founded_on,
            rev_dsl::image_url,
            rev_dsl::image_link_url,
            dsl::id,
            dsl::license,
        ))
        .into_boxed();
    if place_ids.is_empty() {
        log::warn!("Loading all entries at once");
    } else {
        // TODO: Split loading into chunks of fixed size
        log::info!("Loading multiple ({}) entries at once", place_ids.len());
        query = query.filter(dsl::id.eq_any(place_ids));
    }

    let rows = query
        .load::<models::JoinedPlaceRevision>(conn)
        .map_err(from_diesel_err)?;
    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(load_place(conn, row)?);
    }
    Ok(results)
}

fn get_place(conn: &mut SqliteConnection, place_id: &str) -> Result<(Place, ReviewStatus)> {
    let places = get_places(conn, &[place_id])?;
    debug_assert!(places.len() <= 1);
    places.into_iter().next().ok_or(repo::Error::NotFound)
}

fn all_places(conn: &mut SqliteConnection) -> Result<Vec<(Place, ReviewStatus)>> {
    get_places(conn, &[])
}

fn recently_changed_places(
    conn: &mut SqliteConnection,
    params: &RecentlyChangedEntriesParams,
    pagination: &Pagination,
) -> Result<Vec<(Place, ReviewStatus, ActivityLog)>> {
    use schema::{
        place::dsl, place_revision::dsl as rev_dsl, place_revision_review::dsl as review_dsl,
    };

    let mut query = schema::place_revision::table
          .inner_join(
              schema::place::table.on(rev_dsl::parent_rowid
                  .eq(dsl::rowid)
                  .and(rev_dsl::rev.eq(dsl::current_rev))),
          )
          .inner_join(
              schema::place_revision_review::table
                  .on(review_dsl::parent_rowid.eq(rev_dsl::rowid)),
          )
          .select((
              rev_dsl::rowid,
              rev_dsl::rev,
              rev_dsl::created_at,
              rev_dsl::created_by,
              rev_dsl::title,
              rev_dsl::description,
              rev_dsl::lat,
              rev_dsl::lon,
              rev_dsl::street,
              rev_dsl::zip,
              rev_dsl::city,
              rev_dsl::country,
              rev_dsl::state,
              rev_dsl::contact_name,
              rev_dsl::email,
              rev_dsl::phone,
              rev_dsl::homepage,
              rev_dsl::opening_hours,
              rev_dsl::founded_on,
              rev_dsl::image_url,
              rev_dsl::image_link_url,
              dsl::id,
              dsl::license,
              review_dsl::rev,
              review_dsl::created_at,
              review_dsl::created_by,
              review_dsl::status,
              review_dsl::context,
              review_dsl::comment,
          ))
          .order_by(review_dsl::created_at.desc())
          .then_order_by(review_dsl::rev.desc()) // disambiguation of equal time stamps
          .into_boxed();

    // Since (inclusive)
    if let Some(since) = params.since {
        query = query.filter(review_dsl::created_at.ge(since.as_millis()));
    }

    // Until (exclusive)
    if let Some(until) = params.until {
        query = query.filter(review_dsl::created_at.lt(until.as_millis()));
    }

    // Pagination
    let offset = pagination.offset.unwrap_or(0) as i64;
    // SQLite does not support an OFFSET without a LIMIT
    // <https://www.sqlite.org/lang_select.html>
    if let Some(limit) = pagination.limit {
        query = query.limit(limit as i64);
        // Optional OFFSET
        if offset > 0 {
            query = query.offset(offset);
        }
    } else if offset > 0 {
        // Mandatory LIMIT
        query = query.limit(i64::MAX);
        query = query.offset(offset);
    }

    let rows = query
        .load::<models::JoinedPlaceRevisionWithStatusReview>(conn)
        .map_err(from_diesel_err)?;
    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(load_place_with_status_review(conn, row)?);
    }
    Ok(results)
}

fn most_popular_place_revision_tags(
    conn: &mut SqliteConnection,
    params: &MostPopularTagsParams,
    pagination: &Pagination,
) -> Result<Vec<TagFrequency>> {
    // TODO: Diesel 1.4.x does not support the HAVING clause
    // that is required to filter the aggregated column.
    let mut sql = "SELECT tag, COUNT(*) as count \
                     FROM place_revision_tag \
                     WHERE parent_rowid IN \
                     (SELECT rowid FROM place_revision WHERE (parent_rowid, rev) IN (SELECT rowid, current_rev FROM place) AND current_status > 0) \
                     GROUP BY tag"
          .to_string();
    if params.min_count.is_some() || params.max_count.is_some() {
        if let Some(min_count) = params.min_count {
            write!(&mut sql, " HAVING count>={min_count}").unwrap();
            if let Some(max_count) = params.max_count {
                write!(&mut sql, " AND count<={max_count}").unwrap();
            }
        } else if let Some(max_count) = params.max_count {
            write!(&mut sql, " HAVING count<={max_count}").unwrap();
        }
    }
    sql.push_str(" ORDER BY count DESC, tag");
    if let Some(limit) = pagination.limit {
        write!(&mut sql, " LIMIT {limit}").unwrap();
        // LIMIT must precede OFFSET, i.e. OFFSET without LIMIT
        // is not supported!
        let offset = pagination.offset.unwrap_or(0);
        if offset > 0 {
            write!(&mut sql, " OFFSET {offset}").unwrap();
        }
    }
    let rows = diesel::dsl::sql_query(sql)
        .load::<TagCountRow>(conn)
        .map_err(from_diesel_err)?;
    Ok(rows
        .into_iter()
        .map(|row| TagFrequency(row.tag, row.count as TagCount))
        .collect())
}

fn count_places(conn: &mut SqliteConnection) -> Result<usize> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};
    Ok(schema::place_revision::table
        .inner_join(
            schema::place::table.on(rev_dsl::parent_rowid
                .eq(dsl::rowid)
                .and(rev_dsl::rev.eq(dsl::current_rev))),
        )
        .select(diesel::dsl::count(rev_dsl::parent_rowid))
        .filter(rev_dsl::current_status.ge(ReviewStatusPrimitive::from(ReviewStatus::Created)))
        .first::<i64>(conn)
        .map_err(from_diesel_err)? as usize)
}

fn get_place_history(
    conn: &mut SqliteConnection,
    id: &str,
    revision: Option<Revision>,
) -> Result<PlaceHistory> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};

    let mut query = schema::place_revision::table
        .inner_join(schema::place::table.on(rev_dsl::parent_rowid.eq(dsl::rowid)))
        .select((
            rev_dsl::rowid,
            rev_dsl::rev,
            rev_dsl::created_at,
            rev_dsl::created_by,
            rev_dsl::current_status,
            rev_dsl::title,
            rev_dsl::description,
            rev_dsl::lat,
            rev_dsl::lon,
            rev_dsl::street,
            rev_dsl::zip,
            rev_dsl::city,
            rev_dsl::country,
            rev_dsl::state,
            rev_dsl::contact_name,
            rev_dsl::email,
            rev_dsl::phone,
            rev_dsl::homepage,
            rev_dsl::opening_hours,
            rev_dsl::founded_on,
            rev_dsl::image_url,
            rev_dsl::image_link_url,
            dsl::id,
            dsl::license,
        ))
        .filter(dsl::id.eq(id))
        .order_by(rev_dsl::rev.desc())
        .into_boxed();
    if let Some(revision) = revision {
        query = query.filter(rev_dsl::rev.eq(RevisionValue::from(revision) as i64));
    }
    let rows = query
        .load::<models::JoinedPlaceRevision>(conn)
        .map_err(from_diesel_err)?;
    let mut place_history = None;
    let num_revisions = rows.len();
    for row in rows {
        let parent_rowid = row.id;
        let (place, _) = load_place(conn, row)?;
        let (place, place_revision) = place.into();
        if place_history.is_none() {
            place_history = Some(PlaceHistory {
                place,
                revisions: Vec::with_capacity(num_revisions),
            });
        };
        use schema::{place_revision_review::dsl as review_dsl, users::dsl as user_dsl};
        let rows = schema::place_revision_review::table
            .left_outer_join(
                schema::users::table.on(review_dsl::created_by.eq(user_dsl::id.nullable())),
            )
            .select((
                review_dsl::rev,
                review_dsl::created_at,
                review_dsl::created_by,
                user_dsl::email.nullable(),
                review_dsl::status,
                review_dsl::context,
                review_dsl::comment,
            ))
            .filter(review_dsl::parent_rowid.eq(parent_rowid))
            .order_by(review_dsl::rev.desc())
            .load::<models::PlaceReviewedRevision>(conn)
            .map_err(from_diesel_err)?;
        let mut review_logs = Vec::with_capacity(rows.len());
        for row in rows {
            let review_log = ReviewStatusLog {
                revision: Revision::from(row.rev as u64),
                activity: ActivityLog {
                    activity: Activity {
                        at: Timestamp::from_millis(row.created_at),
                        by: row.created_by_email.map(Into::into),
                    },
                    context: row.context,
                    comment: row.comment,
                },
                status: ReviewStatus::try_from(row.status).unwrap(),
            };
            review_logs.push(review_log);
        }
        place_history
            .as_mut()
            .unwrap()
            .revisions
            .push((place_revision, review_logs));
    }
    place_history.ok_or(repo::Error::NotFound)
}

fn load_place_revision(
    conn: &mut SqliteConnection,
    id: &str,
    rev: Revision,
) -> Result<(Place, ReviewStatus)> {
    use schema::{place::dsl, place_revision::dsl as rev_dsl};

    let query = schema::place_revision::table
        .inner_join(
            schema::place::table.on(rev_dsl::parent_rowid
                .eq(dsl::rowid)
                .and(rev_dsl::rev.eq(RevisionValue::from(rev) as i64))),
        )
        .select((
            rev_dsl::rowid,
            rev_dsl::rev,
            rev_dsl::created_at,
            rev_dsl::created_by,
            rev_dsl::current_status,
            rev_dsl::title,
            rev_dsl::description,
            rev_dsl::lat,
            rev_dsl::lon,
            rev_dsl::street,
            rev_dsl::zip,
            rev_dsl::city,
            rev_dsl::country,
            rev_dsl::state,
            rev_dsl::contact_name,
            rev_dsl::email,
            rev_dsl::phone,
            rev_dsl::homepage,
            rev_dsl::opening_hours,
            rev_dsl::founded_on,
            rev_dsl::image_url,
            rev_dsl::image_link_url,
            dsl::id,
            dsl::license,
        ))
        .filter(dsl::id.eq(id));
    let row = query
        .first::<models::JoinedPlaceRevision>(conn)
        .map_err(from_diesel_err)?;
    load_place(conn, row)
}
