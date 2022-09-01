use super::*;

impl<'a> PlaceClearanceRepo for DbReadWrite<'a> {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize> {
        add_pending_clearance_for_places(&mut self.conn.borrow_mut(), org_ids, pending_clearance)
    }
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        count_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id)
    }
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        list_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, pagination)
    }
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>> {
        load_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, place_ids)
    }
    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize> {
        update_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, clearances)
    }
    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        cleanup_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id)
    }
}

impl<'a> PlaceClearanceRepo for DbConnection<'a> {
    fn add_pending_clearance_for_places(
        &self,
        org_ids: &[Id],
        pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize> {
        add_pending_clearance_for_places(&mut self.conn.borrow_mut(), org_ids, pending_clearance)
    }
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        count_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id)
    }
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        list_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, pagination)
    }
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>> {
        load_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, place_ids)
    }
    fn update_pending_clearances_for_places(
        &self,
        org_id: &Id,
        clearances: &[ClearanceForPlace],
    ) -> Result<usize> {
        update_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, clearances)
    }
    fn cleanup_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        cleanup_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id)
    }
}

impl<'a> PlaceClearanceRepo for DbReadOnly<'a> {
    fn add_pending_clearance_for_places(
        &self,
        _org_ids: &[Id],
        _pending_clearance: &PendingClearanceForPlace,
    ) -> Result<usize> {
        unreachable!();
    }
    fn count_pending_clearances_for_places(&self, org_id: &Id) -> Result<u64> {
        count_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id)
    }
    fn list_pending_clearances_for_places(
        &self,
        org_id: &Id,
        pagination: &Pagination,
    ) -> Result<Vec<PendingClearanceForPlace>> {
        list_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, pagination)
    }
    fn load_pending_clearances_for_places(
        &self,
        org_id: &Id,
        place_ids: &[&str],
    ) -> Result<Vec<PendingClearanceForPlace>> {
        load_pending_clearances_for_places(&mut self.conn.borrow_mut(), org_id, place_ids)
    }
    fn update_pending_clearances_for_places(
        &self,
        _org_id: &Id,
        _clearances: &[ClearanceForPlace],
    ) -> Result<usize> {
        unreachable!();
    }
    fn cleanup_pending_clearances_for_places(&self, _org_id: &Id) -> Result<u64> {
        unreachable!();
    }
}

fn add_pending_clearance_for_places(
    conn: &mut SqliteConnection,
    org_ids: &[Id],
    pending_clearance: &PendingClearanceForPlace,
) -> Result<usize> {
    let PendingClearanceForPlace {
        place_id,
        created_at,
        last_cleared_revision,
    } = pending_clearance;
    let place_rowid = resolve_place_rowid(conn, place_id)?;
    let created_at = created_at.as_millis();
    let last_cleared_revision = last_cleared_revision.map(|rev| RevisionValue::from(rev) as i64);
    let mut insert_count = 0;
    for org_id in org_ids {
        let org_rowid = resolve_organization_rowid(conn, org_id)?;
        let insertable = models::NewPendingClearanceForPlace {
            org_rowid,
            place_rowid,
            created_at,
            last_cleared_revision,
        };
        insert_count += diesel::insert_or_ignore_into(schema::organization_place_clearance::table)
            .values(&insertable)
            .execute(conn)
            .map_err(from_diesel_err)?;
    }
    Ok(insert_count)
}

fn count_pending_clearances_for_places(conn: &mut SqliteConnection, org_id: &Id) -> Result<u64> {
    use schema::{organization::dsl as org_dsl, organization_place_clearance::dsl};
    Ok(schema::organization_place_clearance::table
        .filter(
            dsl::org_rowid.eq_any(
                schema::organization::table
                    .select(org_dsl::rowid)
                    .filter(org_dsl::id.eq(org_id.as_str())),
            ),
        )
        .count()
        .get_result::<i64>(conn)
        .map_err(from_diesel_err)? as u64)
}

fn list_pending_clearances_for_places(
    conn: &mut SqliteConnection,
    org_id: &Id,
    pagination: &Pagination,
) -> Result<Vec<PendingClearanceForPlace>> {
    use schema::{
        organization::dsl as org_dsl, organization_place_clearance::dsl, place::dsl as place_dsl,
    };
    let mut query = schema::organization_place_clearance::table
        .inner_join(schema::place::table)
        .select((place_dsl::id, dsl::created_at, dsl::last_cleared_revision))
        .filter(
            dsl::org_rowid.eq_any(
                schema::organization::table
                    .select(org_dsl::rowid)
                    .filter(org_dsl::id.eq(org_id.as_str())),
            ),
        )
        .order_by(dsl::created_at)
        .into_boxed();

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

    Ok(query
        .load::<models::PendingClearanceForPlace>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(Into::into)
        .collect())
}

fn load_pending_clearances_for_places(
    conn: &mut SqliteConnection,
    org_id: &Id,
    place_ids: &[&str],
) -> Result<Vec<PendingClearanceForPlace>> {
    use schema::{
        organization::dsl as org_dsl, organization_place_clearance::dsl, place::dsl as place_dsl,
    };
    Ok(schema::organization_place_clearance::table
        .inner_join(schema::place::table)
        .select((place_dsl::id, dsl::created_at, dsl::last_cleared_revision))
        .filter(
            dsl::org_rowid.eq_any(
                schema::organization::table
                    .select(org_dsl::rowid)
                    .filter(org_dsl::id.eq(org_id.as_str())),
            ),
        )
        .filter(place_dsl::id.eq_any(place_ids))
        .load::<models::PendingClearanceForPlace>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(Into::into)
        .collect())
}

fn update_pending_clearances_for_places(
    conn: &mut SqliteConnection,
    org_id: &Id,
    clearances: &[ClearanceForPlace],
) -> Result<usize> {
    let org_rowid = resolve_organization_rowid(conn, org_id)?;
    let created_at = Timestamp::now().as_millis();
    let mut total_rows_affected = 0;
    for clearance in clearances {
        let ClearanceForPlace {
            place_id,
            cleared_revision,
        } = clearance;
        let (place_rowid, cleared_revision) = if let Some(cleared_revision) = cleared_revision {
            let place_rowid =
                resolve_place_rowid_verify_revision(conn, place_id, *cleared_revision)?;
            (place_rowid, *cleared_revision)
        } else {
            let (place_rowid, current_revision) =
                resolve_place_rowid_with_current_revision(conn, place_id)?;
            (place_rowid, current_revision)
        };
        use schema::{organization::dsl as org_dsl, organization_place_clearance::dsl};
        let last_cleared_revision = Some(RevisionValue::from(cleared_revision) as i64);
        let updatable = models::NewPendingClearanceForPlace {
            org_rowid,
            place_rowid,
            created_at,
            last_cleared_revision,
        };
        let rows_affected = diesel::update(schema::organization_place_clearance::table)
            .set(&updatable)
            .filter(
                dsl::org_rowid.eq_any(
                    schema::organization::table
                        .select(org_dsl::rowid)
                        .filter(org_dsl::id.eq(org_id.as_str())),
                ),
            )
            .filter(dsl::place_rowid.eq(place_rowid))
            .execute(conn)
            .map_err(from_diesel_err)?;
        debug_assert!(rows_affected <= 1);
        total_rows_affected += rows_affected;
    }
    Ok(total_rows_affected)
}

fn cleanup_pending_clearances_for_places(conn: &mut SqliteConnection, org_id: &Id) -> Result<u64> {
    let org_rowid = resolve_organization_rowid(conn, org_id)?;
    use schema::{
        organization_place_clearance::dsl, place::dsl as place_dsl,
        place_revision::dsl as place_rev_dsl,
    };
    let current_place_revisions = schema::place::table
        .inner_join(schema::place_revision::table)
        .filter(place_dsl::current_rev.eq(place_rev_dsl::rev));
    let subselect = current_place_revisions
        .inner_join(
            schema::organization_place_clearance::table.on(dsl::place_rowid.eq(place_dsl::rowid)),
        )
        .select(dsl::rowid)
        .filter(dsl::org_rowid.eq(org_rowid))
        .filter(dsl::last_cleared_revision.eq(place_dsl::current_rev.nullable()));
    // TODO: Diesel 1.4.5 does not allow to use a subselect in the
    // following delete statement and requires to temporarily load
    // the subselect results into memory
    let delete_rowids = subselect.load::<i64>(conn).map_err(from_diesel_err)?;
    let delete_count = diesel::delete(
        schema::organization_place_clearance::table.filter(dsl::rowid.eq_any(delete_rowids)),
    )
    .execute(conn)
    .map_err(from_diesel_err)?;
    Ok(delete_count as u64)
}
