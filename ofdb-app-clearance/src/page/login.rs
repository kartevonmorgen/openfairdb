use gloo_storage::{SessionStorage, Storage};
use seed::{prelude::*, *};

use crate::components::navbar;

#[derive(Debug)]
pub struct Mdl {
    token: String,
    invalid: bool,
    show_password: bool,
    navbar: navbar::Mdl,
}

#[derive(Clone)]
pub enum Msg {
    TogglePasswordVisible,
    Login,
    TokenInput(String),
    Navbar(navbar::Msg),
}

pub fn init(mut url: Url) -> Mdl {
    let invalid = url.next_hash_path_part().unwrap_or("") == crate::HASH_PATH_INVALID;
    Mdl {
        token: String::new(),
        invalid,
        show_password: false,
        navbar: navbar::Mdl {
            login_status: navbar::LoginStatus::LoggedOut,
            menu_is_active: false,
        },
    }
}

pub fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TogglePasswordVisible => {
            mdl.show_password = !mdl.show_password;
            orders.force_render_now();
        }
        Msg::Login => {
            if mdl.show_password {
                orders.send_msg(Msg::TogglePasswordVisible);
                orders.send_msg(Msg::Login);
            } else {
                if let Err(err) = SessionStorage::set(crate::TOKEN_KEY, &mdl.token) {
                    log::debug!("{err}");
                }
                let el = document().get_element_by_id("login-form").unwrap();
                let form = el.dyn_ref::<web_sys::HtmlFormElement>().unwrap();
                if let Err(err) = form.submit() {
                    log::error!("{err:?}");
                }
            }
        }
        Msg::TokenInput(token) => mdl.token = token,
        Msg::Navbar(msg) => {
            navbar::update(msg, &mut mdl.navbar, &mut orders.proxy(Msg::Navbar));
        }
    }
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    let pwfield_type = if mdl.show_password {
        "text"
    } else {
        "password"
    };
    div![
        navbar::view(&mdl.navbar).map_msg(Msg::Navbar),
        main![div![
            C!["container"],
            div![
                C!["section"],
                h2![C!["title"], "Login"],
                if mdl.invalid {
                    div![
                        style! {
                            St::Color => "red",
                            St::PaddingBottom => px(20),
                        },
                        "Your API token is invalid. Please try again"
                    ]
                } else {
                    empty!()
                },
                div![
                    style! {
                        St::PaddingBottom => px(20),
                    },
                    "Enter your organization's API token: ",
                ],
                form![
                    id!("login-form"),
                    attrs! {
                        At::Action => crate::PAGE_URL,
                    },
                    input![
                        C!["input"],
                        attrs! {
                            At::Name => "username",
                            At::Type => "text",
                            At::Value => crate::TITLE,
                        },
                        style! {
                            St::Display => "none",
                        },
                    ],
                    div![
                        C!["field"],
                        label![C!["label"], "Token"],
                        input![
                            C!["input"],
                            attrs! {
                                At::Name => "password",
                                At::Type => pwfield_type,
                            },
                            style! {
                                St::Width => "50%",
                            },
                            input_ev(Ev::Input, Msg::TokenInput),
                        ],
                    ],
                    div![
                        C!["field"],
                        div![
                            C!["control"],
                            label![
                                C!["checkbox"],
                                input![
                                    C!["checkbox"],
                                    attrs! {
                                        At::Type => "checkbox",
                                        At::Checked => mdl.show_password.as_at_value(),
                                    },
                                    ev(Ev::Click, |_| Msg::TogglePasswordVisible),
                                ],
                                " show token",
                            ],
                        ]
                    ],
                    input![
                        C!["button", "is-primary"],
                        attrs! {
                            At::Type => "submit",
                            At::Value => "Login",
                        },
                        ev(Ev::Click, |_| Msg::Login),
                    ],
                ]
            ]
        ]]
    ]
}
