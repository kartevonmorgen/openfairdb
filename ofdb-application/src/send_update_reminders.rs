use super::*;
use ofdb_core::{
    gateways::notify::NotificationGateway, usecases, usecases::EmailReminderFormatter,
};
use std::time::Instant;
use time::Duration;

#[derive(Debug)]
pub struct SendReminderParams {
    pub recipient_role: usecases::RecipientRole,
    pub not_updated_since: Timestamp,
    pub resend_period: Duration,
    pub send_max: u32,
    pub current_time: Timestamp,
    pub token_expire_at: Timestamp,
}

pub fn send_update_reminders<G, F>(
    connections: &sqlite::Connections,
    notification_gateway: &G,
    formatter: &F,
    params: SendReminderParams,
) -> Result<()>
where
    G: NotificationGateway,
    F: EmailReminderFormatter,
{
    let SendReminderParams {
        recipient_role,
        not_updated_since,
        resend_period,
        send_max,
        current_time,
        token_expire_at,
    } = params;

    log::info!("Send update reminders to {recipient_role:?}s for places that were not updated since {not_updated_since:?}");

    // TODO: Safe splitting into multiple transactions.
    // This would require more efforts and intermediate states that are persisted in the database.
    // Using a single transaction would block the DB for a longer time.
    // So for now we make a compromise, unless we are experience frequent issues with duplicate reminders.

    // 1. First use read-only DB connection
    let start_time = Instant::now();
    let unsent_reminders = {
        let conn = connections.shared()?;
        usecases::find_unsent_reminders(
            &conn,
            recipient_role,
            not_updated_since,
            resend_period,
            send_max,
            current_time,
        )?
    };
    log::debug!(
        "Finding unsent reminder emails for {recipient_role:?}s took {}ms",
        start_time.elapsed().as_millis()
    );
    if unsent_reminders.is_empty() {
        log::debug!("There are no unsent reminders to be send");
        return Ok(());
    }

    // 2. Create review tokens
    let unsent_reminders_with_review_tokens =
        usecases::create_reminder_review_tokens(unsent_reminders, token_expire_at);

    connections
        .exclusive()?
        .transaction::<_, _, usecases::Error>(|conn| {
            for (_, token) in &unsent_reminders_with_review_tokens {
                conn.add_review_token(token).map_err(|err| {
                    log::warn!("Failed to save review token: {}", err);
                    err
                })?;
            }
            Ok(())
        })?;

    // 3. Send emails (fire and forget)
    let start_time = Instant::now();

    let unsent_emails =
        ofdb_core::usecases::create_reminder_emails(formatter, unsent_reminders_with_review_tokens);
    let sent_reminders =
        ofdb_core::usecases::send_reminder_emails(notification_gateway, unsent_emails);
    log::debug!(
        "Sending update reminders for {recipient_role:?} stook {}ms",
        start_time.elapsed().as_millis()
    );

    // 4. Remember what emails have been sent.
    let start_time = Instant::now();
    connections.exclusive()?.transaction(|conn| {
        usecases::save_sent_reminders(conn, &sent_reminders, current_time).map_err(|err| {
            log::warn!("Failed to save sent update reminders: {}", err);
            err
        })
    })?;
    log::info!(
        "Saving sent update reminders took {}ms",
        start_time.elapsed().as_millis()
    );

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::prelude::*;
    use ofdb_core::{gateways::notify::NotificationEvent, repositories::*, usecases::Reminder};
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockNotifyGw {
        sent_mails: RefCell<Vec<(Vec<EmailAddress>, EmailContent)>>,
    }

    impl NotificationGateway for MockNotifyGw {
        fn notify(&self, event: NotificationEvent) {
            match event {
                NotificationEvent::ReminderCreated { email, recipients } => {
                    let data = (recipients.to_vec(), email.clone());
                    self.sent_mails.borrow_mut().push(data);
                }
                _ => unreachable!(),
            }
        }
    }

    #[derive(Default)]
    struct MockEmailFormatter;

    impl EmailReminderFormatter for MockEmailFormatter {
        fn format_email(&self, r: &Reminder, _: &ReviewNonce) -> EmailContent {
            EmailContent {
                subject: format!("{}", r.place.id),
                body: format!("{r:?}"),
            }
        }
    }

    fn create_user(fixture: &BackendFixture, role: Role, email: EmailAddress) {
        let user = User {
            email,
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role,
        };
        fixture
            .db_connections
            .exclusive()
            .unwrap()
            .create_user(&user)
            .unwrap();
    }

    fn create_place(
        fixture: &BackendFixture,
        name: &str,
        reviewer_email: EmailAddress,
        review_status: Option<ReviewStatus>,
    ) -> Place {
        // create place
        let place = flows::create_place(
            &fixture.db_connections,
            &mut *fixture.search_engine.borrow_mut(),
            &fixture.notify,
            usecases::NewPlace {
                title: name.into(),
                description: format!("place_{name}"),
                tags: vec![name.to_string()],
                email: Some("owner@example.org".parse().unwrap()),
                ..default_new_place()
            },
            None,
            None,
            &accepted_licenses(),
        )
        .unwrap();

        // create review
        if let Some(status) = review_status {
            let review = usecases::Review {
                context: None,
                comment: None,
                reviewer_email,
                status,
            };
            flows::review_places(
                &fixture.db_connections,
                &mut *fixture.search_engine.borrow_mut(),
                &[place.id.as_str()],
                review,
            )
            .unwrap();
        }

        place
    }

    #[test]
    fn send_update_reminders_to_owners() {
        let fixture = BackendFixture::new();

        let admin_email = "admin@example.org".parse::<EmailAddress>().unwrap();
        create_user(&fixture, Role::Admin, admin_email.clone());

        let old = create_place(
            &fixture,
            "old",
            admin_email.clone(),
            Some(ReviewStatus::Confirmed),
        );
        let _archived = create_place(
            &fixture,
            "archived",
            admin_email.clone(),
            Some(ReviewStatus::Archived),
        );
        let _rejected = create_place(
            &fixture,
            "rejected",
            admin_email.clone(),
            Some(ReviewStatus::Rejected),
        );

        // Resolution of time stamps in the query is 1 sec
        // TODO: Don't waste time by sleeping
        std::thread::sleep(std::time::Duration::from_millis(1001));
        let last_update_time = Timestamp::now();
        let _recent = create_place(&fixture, "recent", admin_email, None);

        let notify_gw = MockNotifyGw::default();
        let email_fmt = MockEmailFormatter::default();

        let not_updated_since = last_update_time;
        let resend_period = Duration::milliseconds(90);
        let current_time = Timestamp::now();
        let token_expire_at = current_time + Duration::seconds(30);

        send_update_reminders(
            &fixture.db_connections,
            &notify_gw,
            &email_fmt,
            SendReminderParams {
                recipient_role: usecases::RecipientRole::Owner,
                not_updated_since,
                resend_period,
                send_max: 10,
                current_time,
                token_expire_at,
            },
        )
        .unwrap();

        let owner_email = "owner@example.org".parse::<EmailAddress>().unwrap();
        assert_eq!(notify_gw.sent_mails.borrow().len(), 1);
        assert_eq!(notify_gw.sent_mails.borrow()[0].0, vec![owner_email]);
        assert_eq!(notify_gw.sent_mails.borrow()[0].1.subject, old.id.as_str());
    }

    #[test]
    fn send_update_reminders_to_scouts() {
        let fixture = BackendFixture::new();

        let admin = "admin@example.org".parse::<EmailAddress>().unwrap();
        let subscribed_scout = "scout-a@example.org".parse::<EmailAddress>().unwrap();
        let passive_scout = "scout-b@example.org".parse::<EmailAddress>().unwrap();
        create_user(&fixture, Role::Admin, admin.clone());
        create_user(&fixture, Role::Scout, subscribed_scout.clone());
        create_user(&fixture, Role::Scout, passive_scout);

        let old = create_place(
            &fixture,
            "old",
            admin.clone(),
            Some(ReviewStatus::Confirmed),
        );
        let _archived = create_place(
            &fixture,
            "archived",
            admin.clone(),
            Some(ReviewStatus::Archived),
        );
        let _rejected = create_place(
            &fixture,
            "rejected",
            admin.clone(),
            Some(ReviewStatus::Rejected),
        );

        let subscription_bbox =
            MapBbox::centered_around(old.location.pos, Distance(10.0), Distance(10.0));

        usecases::subscribe_to_bbox(
            &fixture.db_connections.exclusive().unwrap(),
            subscribed_scout.clone(),
            subscription_bbox,
        )
        .unwrap();

        // Resolution of time stamps in the query is 1 sec
        // TODO: Don't waste time by sleeping
        std::thread::sleep(std::time::Duration::from_millis(1001));
        let last_update_time = Timestamp::now();
        let _recent = create_place(&fixture, "recent", admin, None);

        let notify_gw = MockNotifyGw::default();
        let email_fmt = MockEmailFormatter::default();

        let not_updated_since = last_update_time;
        let resend_period = Duration::milliseconds(90);
        let current_time = Timestamp::now();
        let token_expire_at = current_time + Duration::seconds(30);

        send_update_reminders(
            &fixture.db_connections,
            &notify_gw,
            &email_fmt,
            SendReminderParams {
                recipient_role: usecases::RecipientRole::Scout,
                not_updated_since,
                resend_period,
                send_max: 10,
                current_time,
                token_expire_at,
            },
        )
        .unwrap();

        assert_eq!(notify_gw.sent_mails.borrow().len(), 1);
        assert_eq!(notify_gw.sent_mails.borrow()[0].0, vec![subscribed_scout]);
        assert_eq!(notify_gw.sent_mails.borrow()[0].1.subject, old.id.as_str());
    }
}
