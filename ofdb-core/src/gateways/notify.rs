use ofdb_entities::{
    category::Category,
    email::{EmailAddress, EmailContent},
    event::Event,
    nonce::EmailNonce,
    place::Place,
    user::User,
};

#[derive(Debug)]
pub enum NotificationEvent<'a> {
    PlaceAdded {
        place: &'a Place,
        // TODO: pass affected subscriptions instead of email addresses.
        email_addresses: &'a [EmailAddress],
        // TODO: remove
        all_categories: Vec<Category>,
    },
    PlaceUpdated {
        place: &'a Place,
        // TODO: pass affected subscriptions instead of email addresses.
        email_addresses: &'a [EmailAddress],
        // TODO: remove
        all_categories: Vec<Category>,
    },
    EventAdded {
        event: &'a Event,
        // TODO: pass affected subscriptions instead of email addresses.
        email_addresses: &'a [EmailAddress],
    },
    EventUpdated {
        event: &'a Event,
        // TODO: pass affected subscriptions instead of email addresses.
        email_addresses: &'a [EmailAddress],
    },
    UserRegistered {
        user: &'a User,
        // TODO: don't pass confirmation URL,
        // create it inside the gateway implementation instead.
        confirmation_url: url::Url,
    },
    UserResetPasswordRequested {
        email_nonce: &'a EmailNonce,
    },
    ReminderCreated {
        email: &'a EmailContent,
        recipients: &'a [EmailAddress],
    },
}

impl NotificationEvent<'_> {
    pub const fn kind(&self) -> NotificationType {
        use NotificationType as T;

        match self {
            Self::PlaceAdded { .. } => T::PlaceAdded,
            Self::PlaceUpdated { .. } => T::PlaceUpdated,
            Self::EventAdded { .. } => T::EventAdded,
            Self::EventUpdated { .. } => T::EventUpdated,
            Self::UserRegistered { .. } => T::UserRegistered,
            Self::UserResetPasswordRequested { .. } => T::UserResetPasswordRequested,
            Self::ReminderCreated { .. } => T::ReminderCreated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    PlaceAdded,
    PlaceUpdated,
    EventAdded,
    EventUpdated,
    UserRegistered,
    UserResetPasswordRequested,
    ReminderCreated,
}

pub trait NotificationGateway {
    fn notify(&self, event: NotificationEvent);
}
