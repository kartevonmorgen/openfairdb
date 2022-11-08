use super::*;
use ofdb_core::{gateways::email::EmailGateway, usecases::EmailReminderFormatter};
use time::Duration;

pub fn send_update_reminders<G, F>(
    connections: &sqlite::Connections,
    email_gateway: &G,
    formatter: &F,
    target_contact: usecases::TargetContact,
    unchanged_since: Timestamp,
    resend_period: Duration,
) -> Result<()>
where
    G: EmailGateway,
    F: EmailReminderFormatter,
{
    Ok(connections.exclusive()?.transaction(|conn| {
        usecases::send_update_reminders(
            conn,
            email_gateway,
            formatter,
            target_contact,
            unchanged_since,
            resend_period,
        )
        .map_err(|err| {
            warn!("Failed to send update reminders: {}", err);
            err
        })
    })?)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::prelude::*;
    use ofdb_core::{repositories::*, usecases::Reminder};
    use std::cell::RefCell;

    #[derive(Default)]
    struct MockEmailGw {
        sent_mails: RefCell<Vec<(Vec<Email>, String, String)>>,
    }

    impl EmailGateway for MockEmailGw {
        fn compose_and_send(&self, recipients: &[Email], subject: &str, body: &str) {
            let data = (recipients.to_vec(), subject.to_string(), body.to_string());
            self.sent_mails.borrow_mut().push(data);
        }
    }

    #[derive(Default)]
    struct MockEmailFormatter;

    impl EmailReminderFormatter for MockEmailFormatter {
        fn subject(&self, r: &Reminder) -> String {
            format!("{}", r.place.id)
        }
        fn body(&self, r: &Reminder) -> String {
            format!("{r:?}")
        }
    }

    fn create_user(fixture: &BackendFixture, role: Role, email: Email) {
        let user = User {
            email: email.to_string(),
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
        reviewer_email: Email,
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
                email: Some("owner@example.org".to_string()),
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

        let admin_email = Email::try_from("admin@example.org").unwrap();
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

        let email_gw = MockEmailGw::default();
        let email_fmt = MockEmailFormatter::default();

        let unchanged_since = last_update_time;
        let resend_period = Duration::milliseconds(90);

        send_update_reminders(
            &fixture.db_connections,
            &email_gw,
            &email_fmt,
            usecases::TargetContact::Owner,
            unchanged_since,
            resend_period,
        )
        .unwrap();

        let owner_email = Email::try_from("owner@example.org").unwrap();
        assert_eq!(email_gw.sent_mails.borrow().len(), 1);
        assert_eq!(email_gw.sent_mails.borrow()[0].0, vec![owner_email]);
        assert_eq!(email_gw.sent_mails.borrow()[0].1, old.id.as_str());
    }

    #[test]
    fn send_update_reminders_to_scouts() {
        let fixture = BackendFixture::new();

        let admin = Email::try_from("admin@example.org").unwrap();
        let subscribed_scout = Email::try_from("scout-a@example.org").unwrap();
        let passive_scout = Email::try_from("scout-b@example.org").unwrap();
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
            subscribed_scout.as_str().to_string(),
            subscription_bbox,
        )
        .unwrap();

        // Resolution of time stamps in the query is 1 sec
        // TODO: Don't waste time by sleeping
        std::thread::sleep(std::time::Duration::from_millis(1001));
        let last_update_time = Timestamp::now();
        let _recent = create_place(&fixture, "recent", admin, None);

        let email_gw = MockEmailGw::default();
        let email_fmt = MockEmailFormatter::default();

        let unchanged_since = last_update_time;
        let resend_period = Duration::milliseconds(90);

        send_update_reminders(
            &fixture.db_connections,
            &email_gw,
            &email_fmt,
            usecases::TargetContact::Scout,
            unchanged_since,
            resend_period,
        )
        .unwrap();

        assert_eq!(email_gw.sent_mails.borrow().len(), 1);
        assert_eq!(email_gw.sent_mails.borrow()[0].0, vec![subscribed_scout]);
        assert_eq!(email_gw.sent_mails.borrow()[0].1, old.id.as_str());
    }
}
