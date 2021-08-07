use crate::email::Email;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contact {
    /// The display name of a person
    pub name: Option<String>,

    /// An e-mail address to get in contact
    pub email: Option<Email>,

    /// A phone number to get in contact
    pub phone: Option<String>,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        self.email.is_none() && self.phone.is_none()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn empty_contact() {
        assert!(Contact::default().is_empty());
        let mut c = Contact::default();
        c.email = Some("foo@bar".into());
        assert!(!c.is_empty());
        let mut c = Contact::default();
        c.phone = Some("123".into());
        assert!(!c.is_empty());
    }
}
