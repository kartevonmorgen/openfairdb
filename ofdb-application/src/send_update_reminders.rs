use super::*;
use ofdb_core::{gateways::email::EmailGateway, usecases::EmailReminderFormatter};

pub fn send_update_reminders<G, F>(
    connections: &sqlite::Connections,
    email_gateway: &G,
    formatter: &F,
    target_contact: usecases::TargetContact,
) -> Result<()>
where
    G: EmailGateway,
    F: EmailReminderFormatter,
{
    Ok(connections.exclusive()?.transaction(|conn| {
        usecases::send_update_reminders(conn, email_gateway, formatter, target_contact).map_err(
            |err| {
                warn!("Failed to send update reminders: {}", err);
                err
            },
        )
    })?)
}
