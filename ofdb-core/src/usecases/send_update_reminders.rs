use super::prelude::{Email as EmailAddress, *};
use crate::gateways::email::EmailGateway;
use time::Duration;

pub fn send_update_reminders<R, G, F>(
    repo: &R,
    email_gateway: &G,
    formatter: &F,
    target_contact: TargetContact,
) -> Result<()>
where
    R: PlaceRepo + SubscriptionRepo + ReminderRepo + UserRepo,
    G: EmailGateway,
    F: EmailReminderFormatter,
{
    let unchanged_since = unchanged_since(target_contact);
    let outdated_places = find_places_not_updated_since(repo, unchanged_since)?;
    let unsent_reminders = find_unsent_reminders(repo, outdated_places, target_contact)?;
    let unsent_emails = create_emails(formatter, unsent_reminders);
    send_emails(repo, email_gateway, unsent_emails);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum TargetContact {
    Owner,
    Scout,
}

// TODO: make this configurable
fn unchanged_since(target_contact: TargetContact) -> Timestamp {
    Timestamp::now() - reminder_interval(target_contact)
}

const fn reminder_interval(target_contact: TargetContact) -> Duration {
    match target_contact {
        TargetContact::Owner => ONE_YEAR,
        TargetContact::Scout => FOURHUNDERED_DAYS,
    }
}

const ONE_YEAR: Duration = Duration::days(365);
const FOURHUNDERED_DAYS: Duration = Duration::days(400);

fn find_places_not_updated_since<R>(
    place_repo: &R,
    unchanged_since: Timestamp,
) -> Result<Vec<Place>>
where
    R: PlaceRepo,
{
    let places = place_repo
        .find_places_not_updated_since(unchanged_since)?
        .into_iter()
        .map(|(place, _, _)| place)
        .collect();
    Ok(places)
}

fn find_unsent_reminders<R>(
    repo: &R,
    outdated_places: Vec<Place>,
    target_contact: TargetContact,
) -> Result<Vec<Reminder>>
where
    R: SubscriptionRepo + ReminderRepo + UserRepo,
{
    let mut reminders = vec![];
    let reminder_interval = reminder_interval(target_contact);
    match target_contact {
        TargetContact::Owner => {
            for p in outdated_places {
                if let Some(email) = p.contact_email() {
                    if send_new_reminder(repo, &p.id, email, reminder_interval) {
                        reminders.push(Reminder {
                            recipients: vec![email.clone()],
                            last_change: p.created.at,
                            place: p,
                        });
                    }
                }
            }
        }
        TargetContact::Scout => {
            let subscriptions = repo.all_bbox_subscriptions()?;
            let users = repo.all_users()?;
            for p in outdated_places {
                let scouts = get_scouts_subscribed_to_place(&p, &users, &subscriptions);
                let mut recipients = vec![];
                for s in scouts {
                    let email = EmailAddress::from(s.email.clone());
                    if send_new_reminder(repo, &p.id, &email, reminder_interval) {
                        recipients.push(email);
                    }
                }
                if !recipients.is_empty() {
                    reminders.push(Reminder {
                        recipients,
                        last_change: p.created.at,
                        place: p,
                    });
                }
            }
        }
    }
    Ok(reminders)
}

fn send_new_reminder<R>(
    repo: &R,
    place_id: &Id,
    email: &EmailAddress,
    reminder_interval: Duration,
) -> bool
where
    R: ReminderRepo,
{
    match repo.find_last_sent_reminder(place_id, email).unwrap() {
        Some(last_sent) => {
            let now = Timestamp::now();
            last_sent + reminder_interval > now
        }
        None => true,
    }
}

fn get_scouts_subscribed_to_place<'a>(
    place: &Place,
    users: &'a [User],
    subscriptions: &[BboxSubscription],
) -> Vec<&'a User> {
    let point = place.location.pos;
    let mut subs = subscriptions
        .iter()
        .filter(|s| s.bbox.contains_point(point));
    users
        .iter()
        .filter(|u| u.role == Role::Scout)
        .filter(|u| subs.any(|s| s.user_email == u.email))
        .collect()
}

#[derive(Debug)]
pub struct Reminder {
    pub recipients: Vec<EmailAddress>,
    pub place: Place,
    pub last_change: Timestamp,
}

#[derive(Debug)]
pub struct Email {
    pub subject: String,
    pub body: String,
}

pub trait EmailReminderFormatter {
    fn subject(&self, reminder: &Reminder) -> String;
    fn body(&self, reminder: &Reminder) -> String;
}

fn create_emails<F>(formatter: &F, unsent_reminders: Vec<Reminder>) -> Vec<(Reminder, Email)>
where
    F: EmailReminderFormatter,
{
    unsent_reminders
        .into_iter()
        .map(|r| {
            let subject = formatter.subject(&r);
            let body = formatter.body(&r);
            (r, Email { subject, body })
        })
        .collect()
}

fn send_emails<R, G>(repo: &R, email_gateway: &G, emails: Vec<(Reminder, Email)>)
where
    R: ReminderRepo,
    G: EmailGateway,
{
    for (r, email) in emails {
        let sent_at = Timestamp::now();

        email_gateway.compose_and_send(&r.recipients, &email.subject, &email.body);
        repo.save_sent_reminders(&r.place.id, &r.recipients, sent_at)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ofdb_entities::builders::*;

    #[test]
    fn scouts_subscribed_to_a_place() {
        let pos = MapPoint::from_lat_lng_deg(5.0, 8.0);
        let place = Place::build().pos(pos).finish();
        let covering_bbox = MapBbox::new(
            MapPoint::from_lat_lng_deg(0.0, 0.0),
            MapPoint::from_lat_lng_deg(10.0, 15.0),
        );
        assert!(covering_bbox.contains_point(pos));
        let non_covering_bbox = MapBbox::new(
            MapPoint::from_lat_lng_deg(10.0, 20.0),
            MapPoint::from_lat_lng_deg(30.0, 40.0),
        );
        assert!(!non_covering_bbox.contains_point(pos));

        let password = Password::from("foo".to_string());

        let user = User {
            email: "normal@user.tld".into(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::User,
        };
        let admin = User {
            email: "admin@user.tld".into(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Admin,
        };
        let valid_scout_with_affected_sub = User {
            email: "valid-scout-affected@user.tld".into(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Scout,
        };
        let valid_scout_with_unaffected_sub = User {
            email: "valid-scout-unaffected@user.tld".into(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Scout,
        };
        let invalid_scout_with_affected_sub = User {
            email: "invalid-scout@user.tld".into(),
            password,
            email_confirmed: false,
            role: Role::Scout,
        };

        let user_sub = BboxSubscription {
            id: Id::new(),
            bbox: covering_bbox,
            user_email: user.email.clone(),
        };

        let sub_of_valid_scout_with_affected_sub = BboxSubscription {
            id: Id::new(),
            bbox: covering_bbox,
            user_email: valid_scout_with_affected_sub.email.clone(),
        };
        let sub_of_valid_scout_with_unaffected_sub = BboxSubscription {
            id: Id::new(),
            bbox: non_covering_bbox,
            user_email: valid_scout_with_unaffected_sub.email.clone(),
        };
        let sub_of_invalid_scout_with_affected_sub = BboxSubscription {
            id: Id::new(),
            bbox: covering_bbox,
            user_email: invalid_scout_with_affected_sub.email.clone(),
        };
        let users = vec![
            user,
            admin,
            valid_scout_with_affected_sub.clone(),
            valid_scout_with_unaffected_sub,
            invalid_scout_with_affected_sub,
        ];
        let subscriptions = vec![
            user_sub,
            sub_of_valid_scout_with_affected_sub,
            sub_of_valid_scout_with_unaffected_sub,
            sub_of_invalid_scout_with_affected_sub,
        ];
        let scouts = get_scouts_subscribed_to_place(&place, &users, &subscriptions);
        assert_eq!(scouts.len(), 1);
        assert_eq!(**scouts.get(0).unwrap(), valid_scout_with_affected_sub);
    }
}
