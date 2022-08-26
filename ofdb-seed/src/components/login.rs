//! Login form
//!
//! # Example
//!
//! ```rust
//! use ofdb_seed::components::login;
//! use seed::{*, prelude::*};
//!
//! let mut mdl = login::Mdl::default();
//! mdl.attrs = C!["login-form"];
//!
//! let login = login::view(&mdl);
//!  ```

use seed::{prelude::*, *};

#[derive(Clone)]
pub struct Mdl {
    pub email: String,
    pub password: String,
    pub is_submitting: bool,
    pub attrs: Attrs,
    pub errors: Errors,
    pub labels: Labels,
}

#[derive(Clone, Default)]
pub struct Errors {
    pub form: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone)]
pub struct Labels {
    pub email: String,
    pub email_placeholder: Option<String>,
    pub password: String,
    pub password_placeholder: Option<String>,
    pub login_button: String,
    pub legend: Option<String>,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            email: "E-Mail".to_string(),
            email_placeholder: Some("Enter e-mail address".to_string()),
            password: "Password".to_string(),
            password_placeholder: Some("Enter password".to_string()),
            login_button: "login".to_string(),
            legend: Some("Login".to_string()),
        }
    }
}

impl Default for Mdl {
    fn default() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            is_submitting: false,
            attrs: attrs! {},
            errors: Default::default(),
            labels: Default::default(),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    EmailChanged(String),
    PasswordChanged(String),
    Submit,
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    form![
        attrs! {
            At::Action => "javascript:void(0);";
        },
        &mdl.attrs,
        if let Some(msg) = &mdl.errors.form {
            div![C!["error"], msg]
        } else {
            empty!()
        },
        fieldset![
            if let Some(l) = &mdl.labels.legend {
                legend![l]
            } else {
                empty!()
            },
            label![
                span![&mdl.labels.email],
                input![
                    input_ev(Ev::Input, Msg::EmailChanged),
                    attrs! {
                        At::Type => "email";
                        At::Name => "email";
                        At::Required => true.as_at_value();
                        At::Value => mdl.email;
                        At::Disabled => mdl.is_submitting.as_at_value();
                    },
                    if let Some(p) = &mdl.labels.email_placeholder {
                        attrs! {
                            At::Placeholder => p;
                        }
                    } else {
                        attrs! {}
                    }
                ],
                if let Some(msg) = &mdl.errors.email {
                    div![C!["error"], msg]
                } else {
                    empty!()
                }
            ],
            label![
                span![&mdl.labels.password],
                input![
                    input_ev(Ev::Input, Msg::PasswordChanged),
                    attrs! {
                        At::Type => "password";
                        At::Name => "password";
                        At::Required => true.as_at_value();
                        At::Value => mdl.password;
                        At::Disabled => mdl.is_submitting.as_at_value();
                    },
                    if let Some(p) = &mdl.labels.password_placeholder {
                        attrs! {
                            At::Placeholder => p;
                        }
                    } else {
                        attrs! {}
                    }
                ],
                if let Some(msg) = &mdl.errors.password {
                    div![C!["error"], msg]
                } else {
                    empty!()
                }
            ]
        ],
        input![
            ev(Ev::Click, |_| Msg::Submit),
            attrs! {
                At::Value => mdl.labels.login_button;
                At::Type => "submit";
                At::Disabled => mdl.is_submitting.as_at_value();
            }
        ]
    ]
}
