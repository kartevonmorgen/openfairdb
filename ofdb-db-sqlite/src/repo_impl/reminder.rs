use super::*;

impl ReminderRepo for DbReadWrite<'_> {
    fn find_last_sent_reminder(
        &self,
        place_id: &Id,
        email: &EmailAddress,
    ) -> Result<Option<Timestamp>> {
        find_last_sent_reminder(&mut self.conn.borrow_mut(), place_id, email)
    }
    fn save_sent_reminders(
        &self,
        place_id: &Id,
        recipients: &[EmailAddress],
        sent_at: Timestamp,
    ) -> Result<()> {
        save_sent_reminders(&mut self.conn.borrow_mut(), place_id, recipients, sent_at)
    }
}

impl ReminderRepo for DbConnection<'_> {
    fn find_last_sent_reminder(
        &self,
        place_id: &Id,
        email: &EmailAddress,
    ) -> Result<Option<Timestamp>> {
        find_last_sent_reminder(&mut self.conn.borrow_mut(), place_id, email)
    }
    fn save_sent_reminders(
        &self,
        place_id: &Id,
        recipients: &[EmailAddress],
        sent_at: Timestamp,
    ) -> Result<()> {
        save_sent_reminders(&mut self.conn.borrow_mut(), place_id, recipients, sent_at)
    }
}

impl ReminderRepo for DbReadOnly<'_> {
    fn find_last_sent_reminder(
        &self,
        place_id: &Id,
        email: &EmailAddress,
    ) -> Result<Option<Timestamp>> {
        find_last_sent_reminder(&mut self.conn.borrow_mut(), place_id, email)
    }
    fn save_sent_reminders(
        &self,
        _place_id: &Id,
        _recipients: &[EmailAddress],
        _sent_at: Timestamp,
    ) -> Result<()> {
        unreachable!();
    }
}

fn save_sent_reminders(
    conn: &mut SqliteConnection,
    place_id: &Id,
    recipients: &[EmailAddress],
    sent_at: Timestamp,
) -> Result<()> {
    let place_rowid = resolve_place_rowid(conn, place_id)?;
    let sent_at = sent_at.as_millis();
    let mut insert_count = 0;
    for sent_to_email in recipients {
        let insertable = models::NewSentReminder {
            place_rowid,
            sent_at,
            sent_to_email: sent_to_email.as_str(),
        };
        insert_count += diesel::insert_or_ignore_into(schema::sent_reminders::table)
            .values(&insertable)
            .execute(conn)
            .map_err(from_diesel_err)?;
    }
    debug_assert_eq!(insert_count, recipients.len());
    Ok(())
}

fn find_last_sent_reminder(
    conn: &mut SqliteConnection,
    place_id: &Id,
    email: &EmailAddress,
) -> Result<Option<Timestamp>> {
    let place_rowid = resolve_place_rowid(conn, place_id)?;
    use schema::{place::dsl as place_dsl, sent_reminders::dsl};

    // TODO: use GROUP BY + MAX
    // It would be better to filter for MAX(sent_at) that results in only a single value.

    let query = schema::sent_reminders::table
        .inner_join(schema::place::table)
        .select((place_dsl::id, dsl::sent_at, dsl::sent_to_email))
        .filter(place_dsl::rowid.eq(place_rowid))
        .filter(dsl::sent_to_email.eq(email.as_str()))
        .order_by(dsl::sent_at.desc());
    let mut iter = query
        .load::<models::SentReminder>(conn)
        .map_err(from_diesel_err)?
        .into_iter()
        .map(|r| r.sent_at)
        .map(|millis| Timestamp::try_from_millis(millis).unwrap());
    let first = iter.next();
    debug_assert!(iter.next().is_none());
    Ok(first)
}
