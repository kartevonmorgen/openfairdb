use super::*;

impl<'a> ReminderRepo for DbReadWrite<'a> {
    fn find_last_sent_reminder(&self, _place_id: &Id, _email: &Email) -> Result<Option<Timestamp>> {
        todo!()
    }
    fn save_sent_reminders(
        &self,
        _place_id: &Id,
        _recipients: &[Email],
        _sent_at: Timestamp,
    ) -> Result<()> {
        todo!()
    }
}

impl<'a> ReminderRepo for DbConnection<'a> {
    fn find_last_sent_reminder(&self, _place_id: &Id, _email: &Email) -> Result<Option<Timestamp>> {
        todo!()
    }
    fn save_sent_reminders(
        &self,
        _place_id: &Id,
        _recipients: &[Email],
        _sent_at: Timestamp,
    ) -> Result<()> {
        todo!()
    }
}

impl<'a> ReminderRepo for DbReadOnly<'a> {
    fn find_last_sent_reminder(&self, _place_id: &Id, _email: &Email) -> Result<Option<Timestamp>> {
        todo!()
    }
    fn save_sent_reminders(
        &self,
        _place_id: &Id,
        _recipients: &[Email],
        _sent_at: Timestamp,
    ) -> Result<()> {
        unreachable!();
    }
}
