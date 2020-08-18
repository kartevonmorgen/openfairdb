use seed::{prelude::*, *};

#[derive(Debug)]
pub struct Mdl {
    token: String,
    invalid: bool,
    show_password: bool,
}

#[derive(Clone)]
pub enum Msg {
    TogglePasswordVisible,
    Login,
    TokenInput(String),
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
                if let Err(err) = SessionStorage::insert(crate::TOKEN_KEY, &mdl.token) {
                    log!(err);
                }
                let el = document().get_element_by_id("login-form").unwrap();
                let form = el.dyn_ref::<web_sys::HtmlFormElement>().unwrap();
                if let Err(err) = form.submit() {
                    error!(err);
                }
            }
        }
        Msg::TokenInput(token) => mdl.token = token,
    }
}

pub fn init(mut url: Url) -> Option<Mdl> {
    let invalid = url.next_hash_path_part().unwrap_or("") == "invalid";
    Some(Mdl {
        token: String::new(),
        invalid,
        show_password: false,
    })
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    let pwfield_type = if mdl.show_password {
        "text"
    } else {
        "password"
    };
    div![
        h1![crate::TITLE],
        h2!["Login"],
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
                attrs! {
                    At::Name => "username",
                    At::Type => "text",
                    At::Value => crate::TITLE,
                },
                style! {
                    St::Display => "none",
                },
            ],
            input![
                attrs! {
                    At::Name => "password",
                    At::Type => pwfield_type,
                },
                style! {
                    St::Width => "50%",
                },
                input_ev(Ev::Input, Msg::TokenInput),
            ],
            " ",
            input![
                attrs! {
                    At::Type => "submit",
                    At::Value => "Login",
                },
                simple_ev(Ev::Click, Msg::Login),
            ],
            div![
                input![
                    attrs! {
                        At::Id => "pw-visi",
                        At::Type => "checkbox",
                        At::Checked => mdl.show_password.as_at_value(),
                    },
                    simple_ev(Ev::Click, Msg::TogglePasswordVisible),
                ],
                label![
                    attrs! {
                        At::For => "pw-visi",
                    },
                    "Show token",
                ],
            ],
        ]
    ]
}
