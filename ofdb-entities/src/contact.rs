use crate::email::Email;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contact {
    pub email: Option<Email>,
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
        assert_eq!(c.is_empty(), false);
        let mut c = Contact::default();
        c.phone = Some("123".into());
        assert_eq!(c.is_empty(), false);
    }
}
