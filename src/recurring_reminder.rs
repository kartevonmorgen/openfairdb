use crate::{config, gateways};

use ofdb_application::prelude::send_update_reminders;
use ofdb_core::{entities::Timestamp, usecases::RecipientRole};
use ofdb_db_sqlite::Connections;
use ofdb_gateways::user_communication::ReminderFormatter;
use time::Duration;

pub async fn run(
    connections: &Connections,
    email_gateway_cfg: Option<config::EmailGateway>,
    cfg: config::Reminders,
) {
    if cfg.send_to.is_empty() {
        return;
    }

    let mut interval = tokio::time::interval(cfg.task_interval_time);
    let email_gw = gateways::email_gateway(email_gateway_cfg);

    loop {
        interval.tick().await;

        for recipient_role in &cfg.send_to {
            let formatter = ReminderFormatter::new(*recipient_role);
            let resend_period = resend_period(*recipient_role, &cfg);
            let not_updated_since = Timestamp::now() - resend_period;

            if let Err(err) = send_update_reminders(
                connections,
                &email_gw,
                &formatter,
                *recipient_role,
                not_updated_since,
                resend_period,
                cfg.send_max,
            ) {
                log::warn!("Update reminders could not be sent: {err}");
            }
        }
    }
}

fn resend_period(recipient_role: RecipientRole, cfg: &config::Reminders) -> Duration {
    let duration = match recipient_role {
        RecipientRole::Owner => cfg.owners.not_updated_for,
        RecipientRole::Scout => cfg.scouts.not_updated_for,
    };
    Duration::try_from(duration).expect("Resend period")
}
