use time::Duration;

use ofdb_application::prelude::{send_update_reminders, SendReminderParams};
use ofdb_core::{entities::Timestamp, usecases::RecipientRole};
use ofdb_db_sqlite::Connections;
use ofdb_gateways::notify::Notify;
use ofdb_gateways::user_communication::ReminderFormatter;

use crate::config;

pub async fn run(connections: Connections, notification_gw: Notify, cfg: config::Reminders) {
    if cfg.send_to.is_empty() {
        log::info!("No recipient defined in `send_to`: do not send recurring reminders");
        return;
    }

    let mut interval = tokio::time::interval(cfg.task_interval_time);
    let token_expire_in = Duration::try_from(cfg.token_expire_in).expect("Token expiring duration");

    log::info!(
        "Send recurring reminders to {:?} (interval = {interval:?})",
        cfg.send_to
    );

    loop {
        for recipient_role in &cfg.send_to {
            let recipient_role = *recipient_role;
            let formatter = ReminderFormatter::new(recipient_role);
            let resend_period = resend_period(recipient_role, &cfg);
            let current_time = Timestamp::now();
            let not_updated_since = current_time - resend_period;
            let token_expire_at = current_time + token_expire_in;
            let bcc = &cfg.send_bcc;

            let params = SendReminderParams {
                recipient_role,
                not_updated_since,
                resend_period,
                send_max: cfg.send_max,
                current_time,
                token_expire_at,
                bcc,
            };

            if let Err(err) =
                send_update_reminders(&connections, &notification_gw, &formatter, params)
            {
                log::warn!("Update reminders could not be sent: {err}");
            }
        }
        interval.tick().await;
    }
}

fn resend_period(recipient_role: RecipientRole, cfg: &config::Reminders) -> Duration {
    let duration = match recipient_role {
        RecipientRole::Owner => cfg.owners.not_updated_for,
        RecipientRole::Scout => cfg.scouts.not_updated_for,
    };
    Duration::try_from(duration).expect("Resend period")
}
