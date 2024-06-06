use super::prelude::*;
use crate::{
    gateways::notify::{NotificationEvent, NotificationGateway},
    repositories,
};
use anyhow::anyhow;
use std::{ops::Not, time::Instant};
use time::Duration;

const DEFAULT_STEP_WIDTH: u64 = 100;

pub fn find_unsent_reminders<R>(
    repo: &R,
    recipient_role: RecipientRole,
    not_updated_since: Timestamp,
    resend_period: Duration,
    send_max: u32,
    now: Timestamp,
) -> Result<Vec<Reminder>>
where
    R: PlaceRepo + SubscriptionRepo + ReminderRepo + UserRepo,
{
    let limit = Some(u64::from(send_max));
    let mut offset = None;
    let mut unsent_reminders = vec![];

    loop {
        let pagination = Pagination { offset, limit };
        let outdated_places = find_places_not_updated_since(repo, not_updated_since, &pagination)?;
        log::debug!("Found {} outdated places", outdated_places.len());
        debug_assert!(outdated_places.len() <= send_max.try_into().unwrap());
        if outdated_places.is_empty() {
            break; // Nothing to do, everything is up to date :)
        }
        let unsent = find_unsent_reminders_for_outdated_places(
            repo,
            outdated_places,
            recipient_role,
            resend_period,
            now,
        )?;
        if unsent.is_empty() {
            // All reminders were sent for these outdated places,
            // so look for more by increasing the offset
            let new_offset = offset.unwrap_or(0).checked_add(DEFAULT_STEP_WIDTH).ok_or_else(||
            // We should never reach u64::MAX
            repositories::Error::Other(anyhow!("The offset to find outdated places exceeds maximum"))
          )?;
            offset = Some(new_offset);
            continue;
        } else {
            unsent_reminders = unsent;
            break;
        }
    }

    log::debug!("Found {} unsent reminders", unsent_reminders.len());
    Ok(unsent_reminders)
}

pub fn send_reminder_emails<G>(notify: &G, emails: Vec<(Reminder, EmailContent)>) -> Vec<Reminder>
where
    G: NotificationGateway,
{
    emails
        .into_iter()
        .map(|(r, email)| {
            let event = NotificationEvent::ReminderCreated {
                recipients: &r.recipients,
                email: &email,
            };
            notify.notify(event);
            r
        })
        .collect()
}

pub fn save_sent_reminders<R>(
    repo: &R,
    sent_reminders: &[Reminder],
    sent_at: Timestamp,
) -> Result<()>
where
    R: ReminderRepo,
{
    for reminder in sent_reminders {
        repo.save_sent_reminders(&reminder.place.id, &reminder.recipients, sent_at)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecipientRole {
    Owner,
    Scout,
}

fn find_places_not_updated_since<R>(
    place_repo: &R,
    not_updated_since: Timestamp,
    pagination: &Pagination,
) -> Result<Vec<Place>>
where
    R: PlaceRepo,
{
    let start_time = Instant::now();
    let places = place_repo
        .find_places_not_updated_since(not_updated_since, pagination)?
        .into_iter()
        .map(|(place, _)| place)
        .collect();
    log::debug!(
        "find places not updated since {not_updated_since:?} took {}ms",
        start_time.elapsed().as_millis()
    );
    Ok(places)
}

fn find_unsent_reminders_for_outdated_places<R>(
    repo: &R,
    outdated_places: Vec<Place>,
    recipient_role: RecipientRole,
    resend_period: Duration,
    now: Timestamp,
) -> Result<Vec<Reminder>>
where
    R: SubscriptionRepo + ReminderRepo + UserRepo,
{
    // TODO:
    // Use SubscriptionRepo::bbox_subscriptions_affected_by_place and as soon as it is implemented
    // and combine it with UserRepo::get_user_by_email.

    let start_time = Instant::now();
    let subscriptions = repo.all_bbox_subscriptions()?;
    let users = repo.all_users()?;

    let reminders = outdated_places
        .into_iter()
        .filter_map(|place| {
            match recipient_role {
                RecipientRole::Owner => contact_email_addresses(repo, &place, resend_period, now),
                RecipientRole::Scout => {
                    scout_email_addresses(repo, &place, &users, &subscriptions, resend_period, now)
                }
            }
            .map(|recipients| (place, recipients))
        })
        .map(|(place, recipients)| Reminder {
            recipients,
            last_change: place.created.at,
            place,
        })
        .collect();
    log::debug!(
        "find unsent reminders took {}ms",
        start_time.elapsed().as_millis()
    );
    Ok(reminders)
}

fn scout_email_addresses<R>(
    repo: &R,
    place: &Place,
    users: &[User],
    subscriptions: &[BboxSubscription],
    resend_period: Duration,
    now: Timestamp,
) -> Option<Vec<EmailAddress>>
where
    R: ReminderRepo,
{
    let scouts = get_scouts_subscribed_to_place(place, users, subscriptions);
    let email_addresses = scouts
        .iter()
        .filter_map(|scout| {
            sending_new_reminder_is_needed(repo, &place.id, &scout.email, now, resend_period)
                .then_some(&scout.email)
                .cloned()
        })
        .collect::<Vec<_>>();
    email_addresses.is_empty().not().then_some(email_addresses)
}

fn contact_email_addresses<R>(
    repo: &R,
    place: &Place,
    resend_period: Duration,
    now: Timestamp,
) -> Option<Vec<EmailAddress>>
where
    R: ReminderRepo,
{
    place.contact_email().and_then(|email| {
        sending_new_reminder_is_needed(repo, &place.id, email, now, resend_period)
            .then(|| vec![email.clone()])
    })
}

fn get_scouts_subscribed_to_place<'a>(
    place: &Place,
    users: &'a [User],
    subscriptions: &[BboxSubscription],
) -> Vec<&'a User> {
    let subscriptions = bbox_subscriptions_affected_by_place(subscriptions, place);
    get_subscribed_scouts(users, &subscriptions)
}

fn bbox_subscriptions_affected_by_place<'a>(
    subscriptions: &'a [BboxSubscription],
    place: &Place,
) -> Vec<&'a BboxSubscription> {
    let point = place.location.pos;
    subscriptions
        .iter()
        .filter(|s| s.bbox.contains_point(point))
        .collect()
}

fn get_subscribed_scouts<'a>(
    users: &'a [User],
    subscriptions: &[&BboxSubscription],
) -> Vec<&'a User> {
    users
        .iter()
        .filter(|u| u.role == Role::Scout)
        .filter(|u| u.email_confirmed)
        .filter(|u| subscriptions.iter().any(|s| s.user_email == u.email))
        .collect()
}

fn sending_new_reminder_is_needed<R>(
    repo: &R,
    place_id: &Id,
    email: &EmailAddress,
    current_date_time: Timestamp,
    resend_period: Duration,
) -> bool
where
    R: ReminderRepo,
{
    match repo.find_last_sent_reminder(place_id, email) {
        Ok(Some(last_sent)) => last_sent + resend_period < current_date_time,
        Ok(None) => true,
        Err(err) => {
            log::warn!("Unable to find last sent reminder for {place_id} and {email}: {err}");
            false
        }
    }
}

#[derive(Debug)]
pub struct Reminder {
    pub recipients: Vec<EmailAddress>,
    pub place: Place,
    pub last_change: Timestamp,
}

pub trait EmailReminderFormatter {
    fn format_email(&self, reminder: &Reminder, review_nonce: &ReviewNonce) -> EmailContent;
}

pub fn create_reminder_review_tokens(
    reminders: Vec<Reminder>,
    expires_at: Timestamp,
) -> Vec<(Reminder, ReviewToken)> {
    reminders
        .into_iter()
        .map(|reminder| {
            let place_id = reminder.place.id.clone();
            let place_revision = reminder.place.revision;
            let nonce = Nonce::new();
            let review_nonce = ReviewNonce {
                place_id,
                place_revision,
                nonce,
            };
            let review_token = ReviewToken {
                expires_at,
                review_nonce,
            };
            (reminder, review_token)
        })
        .collect()
}

pub fn create_reminder_emails<F>(
    formatter: &F,
    unsent_reminders: Vec<(Reminder, ReviewToken)>,
) -> Vec<(Reminder, EmailContent)>
where
    F: EmailReminderFormatter,
{
    unsent_reminders
        .into_iter()
        .map(|(reminder, token)| {
            let email = formatter.format_email(&reminder, &token.review_nonce);
            (reminder, email)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usecases::{self, tests::*, NewPlace};
    use ofdb_entities::builders::*;
    use time::Duration;

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
            email: "normal@user.tld".parse().unwrap(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::User,
        };
        let admin = User {
            email: "admin@user.tld".parse().unwrap(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Admin,
        };
        let valid_scout_with_affected_sub = User {
            email: "valid-scout-affected@user.tld".parse().unwrap(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Scout,
        };
        let valid_scout_with_unaffected_sub = User {
            email: "valid-scout-unaffected@user.tld".parse().unwrap(),
            password: password.clone(),
            email_confirmed: true,
            role: Role::Scout,
        };
        let invalid_scout_with_affected_sub = User {
            email: "invalid-scout@user.tld".parse().unwrap(),
            password,
            email_confirmed: false,
            role: Role::Scout,
        };

        let affected_user_sub = BboxSubscription {
            id: Id::new(),
            bbox: covering_bbox,
            user_email: user.email.clone(),
        };

        let affected_sub_of_valid_scout = BboxSubscription {
            id: Id::new(),
            bbox: covering_bbox,
            user_email: valid_scout_with_affected_sub.email.clone(),
        };
        let unaffected_sub_of_valid_scout = BboxSubscription {
            id: Id::new(),
            bbox: non_covering_bbox,
            user_email: valid_scout_with_unaffected_sub.email.clone(),
        };
        let affected_sub_of_invalid_scout = BboxSubscription {
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
            affected_user_sub,
            affected_sub_of_valid_scout,
            unaffected_sub_of_valid_scout,
            affected_sub_of_invalid_scout,
        ];
        let affected_subs = bbox_subscriptions_affected_by_place(&subscriptions, &place);
        assert_eq!(affected_subs.len(), 3);
        let scouts = get_subscribed_scouts(&users, &affected_subs);
        assert_eq!(scouts.len(), 1);
        let scouts = get_scouts_subscribed_to_place(&place, &users, &subscriptions);
        assert_eq!(scouts.len(), 1);
        assert_eq!(**scouts.first().unwrap(), valid_scout_with_affected_sub);
    }

    #[test]
    fn check_if_reminder_needs_to_be_resent() {
        let mock_db = MockDb::default();
        let accepted_licenses = accepted_licenses();
        let license = accepted_licenses.iter().next().unwrap();
        let new_place = NewPlace::build().license(license).finish();
        let storable =
            usecases::prepare_new_place(&mock_db, new_place, None, None, &accepted_licenses)
                .unwrap();
        let (place, _) = usecases::store_new_place(&mock_db, storable).unwrap();
        assert_eq!(mock_db.entries.borrow().len(), 1);

        let email_address = "scout@example.com".parse::<EmailAddress>().unwrap();
        let resent_period = Duration::seconds(5);
        let now = Timestamp::now();

        assert!(sending_new_reminder_is_needed(
            &mock_db,
            &place.id,
            &email_address,
            now,
            resent_period
        ));
        mock_db
            .save_sent_reminders(&place.id, &[email_address.clone()], now)
            .unwrap();
        assert!(!sending_new_reminder_is_needed(
            &mock_db,
            &place.id,
            &email_address,
            now,
            resent_period
        ));
    }
}
