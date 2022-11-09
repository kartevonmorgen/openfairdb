use ofdb_application::prelude::send_update_reminders;
use ofdb_core::{entities::Timestamp, usecases::RecipientRole};
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

        let recipient_role = RecipientRole::Owner;
        let not_updated_since = not_updated_since(recipient_role);
        let resend_period = resend_period(recipient_role);

        if let Err(err) = send_update_reminders(
            connections,
            &email_gw,
            &formatter,
            recipient_role,
            not_updated_since,
            resend_period,
        ) {
            log::warn!("Update reminders could not be sent: {err}");
        }
    }
}

const fn resend_period(recipient_role: RecipientRole) -> Duration {
    match recipient_role {
        RecipientRole::Owner => ONE_YEAR,
        RecipientRole::Scout => FOURHUNDERED_DAYS,
    }
}

// TODO: make this configurable
fn not_updated_since(recipient_role: RecipientRole) -> Timestamp {
    Timestamp::now() - resend_period(recipient_role)
}
