use ofdb_entities::{event::*, place::*, subscription::*, geo::*, email::*};

pub type Subject = String;
pub type Body = String;

// TODO: move into ofdb-core crate
/// The kind of notification.
#[derive(Debug, PartialEq)]
pub enum Notification {
    PlaceCreated(Place),
    PlaceUpdated(Place), // TODO: Do we need both the previous and updated version?
    EventCreated(Event),
    EventUpdated(Event), // TODO: Do we need both the previous and updated version?
    SingleEmail(Email, Subject, Body),  // TODO: Use a struct like { to: Email, subject: String, ... }
}

// TODO: move into ofdb-core crate
/// A subsystem that manages everything concerning
/// notifications.
pub trait NotificationManager {
    fn notify(&self, n: Notification);
    // TODO: fn subscribe(&self, user: &User, bbox: MapBbox) -> Result<Id,SubscriptionError>;
}

/// A email notification manager.
pub struct Manager {
    pub api_key: String,
    pub domain: String,
    pub from_email: Email,
    pub subscriptions: Vec<BboxSubscription>
}

impl NotificationManager for Manager {
    fn notify(&self, n: Notification) {
        // TODO:
        // Send notifications from a separate thread and
        // return immediately from the trait method.
        match n {
            Notification::PlaceCreated(ref _place) => {
                let client = reqwest::Client::new();
                let params = [
                    ("from", &*self.from_email),
                    // TODO: ("to", EMAIL ADDRESS),
                    // TODO: ("subject", SUBJECT TEXT FROM PLACE ),
                    // TODO: ("text", TEXT FROM PLACE),
                ];
                let res = client
                    .post(&self.api_url())
                    .form(&params)
                    .basic_auth("api", Some(&self.api_key))
                    .send();
                match res {
                    Ok(res) => {
                        if res.status().is_success() {
                            debug!("Mail provider response: {:#?}", res);
                        } else {
                            error!("Could not send email: response status: {:?}", res.status());
                            // TODO: We should distinguish between a technical failure (Err, see below)
                            // and an error response (here). Both log messages start with the same text.
                        }
                    }
                    Err(err) => {
                        error!("Could not send email: {}", err);
                    }
                }
            }
            _ => todo!(),
        }
    }
}

impl Manager {
    fn api_url(&self) -> String {
        format!("https://api.mailgun.net/v3/{}/messages", self.domain)
    }
}
