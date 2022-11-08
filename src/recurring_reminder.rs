use ofdb_application::prelude::send_update_reminders;
use ofdb_core::{entities::Timestamp, usecases::TargetContact};
use ofdb_db_sqlite::Connections;
use ofdb_gateways::user_communication::ReminderFormatter;
use std::time::Duration as StdDuration;
use time::Duration;

const ONE_YEAR: Duration = Duration::days(365);
const FOURHUNDERED_DAYS: Duration = Duration::days(400);
const TASK_INTERVAL_DURATION: StdDuration = StdDuration::from_secs(60 * 60 * 24);

pub async fn run(connections: &Connections) {
    let mut interval = tokio::time::interval(TASK_INTERVAL_DURATION);
    let email_gw = crate::gateways::email_gateway();
    let formatter = ReminderFormatter::default();

    loop {
        interval.tick().await;

        let target_contact = TargetContact::Owner;
        let unchanged_since = unchanged_since(target_contact);
        let resend_period = resend_period(target_contact);

        if let Err(err) = send_update_reminders(
            connections,
            &email_gw,
            &formatter,
            target_contact,
            unchanged_since,
            resend_period,
        ) {
            log::warn!("Update reminders could not be sent: {err}");
        }
    }
}

const fn resend_period(target_contact: TargetContact) -> Duration {
    match target_contact {
        TargetContact::Owner => ONE_YEAR,
        TargetContact::Scout => FOURHUNDERED_DAYS,
    }
}

// TODO: make this configurable
fn unchanged_since(target_contact: TargetContact) -> Timestamp {
    Timestamp::now() - resend_period(target_contact)
}
