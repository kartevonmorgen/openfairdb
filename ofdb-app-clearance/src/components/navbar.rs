use seed::{prelude::*, *};

#[derive(Debug)]
pub struct Mdl {
    pub login_status: LoginStatus,
    pub menu_is_active: bool,
}

#[derive(Debug, Clone)]
pub enum Msg {
    ToggleMenu,
    Brand,
    Login,
    Logout,
}

#[derive(Debug)]
pub enum LoginStatus {
    LoggedIn,
    LoggedOut,
}

pub fn update(msg: Msg, mdl: &mut Mdl, _: &mut impl Orders<Msg>) {
    #[allow(clippy::single_match)]
    match msg {
        Msg::ToggleMenu => {
            mdl.menu_is_active = !mdl.menu_is_active;
        }
        _ => {
            // should be handled by the parent component
        }
    }
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    nav![
        C!["navbar"],
        attrs! {
            At::Custom("role".into()) => "navigation";
            At::Custom("aria-label".into()) => "main navigation";
        },
        div![
            C!["navbar-brand"],
            a![
                C!["navbar-item"],
                ev(Ev::Click, |_| Msg::Brand),
                crate::TITLE
            ],
            burger_menu(mdl.menu_is_active, Msg::ToggleMenu)
        ],
        div![
            if mdl.menu_is_active {
                C!["navbar-menu", "is-active"]
            } else {
                C!["navbar-menu"]
            },
            div![C!["navbar-start"]],
            div![
                C!["navbar-end"],
                div![
                    C!["navbar-item"],
                    div![
                        C!["buttons"],
                        match &mdl.login_status {
                            LoginStatus::LoggedIn => {
                                logout_button(Msg::Logout)
                            }
                            LoginStatus::LoggedOut => {
                                login_button(Msg::Login)
                            }
                        }
                    ]
                ]
            ]
        ]
    ]
}

fn burger_menu<M: Clone + 'static>(active: bool, msg: M) -> Node<M> {
    let span = span![attrs! {At::Custom("aria-hidden".into())=>"true";}];
    a![
        ev(Ev::Click, |_| msg),
        if active {
            C!["navbar-burger", "is-active"]
        } else {
            C!["navbar-burger"]
        },
        attrs! {
            At::Custom("role".into()) => "button";
            At::Custom("aria-label".into()) =>"menu";
            At::Custom("aria-expanded".into()) =>"false";
        },
        &span,
        &span,
        &span,
    ]
}

fn login_button<M: Clone + 'static>(msg: M) -> Node<M> {
    a![C!["button", "is-light"], ev(Ev::Click, |_| msg), "Log in"]
}

fn logout_button<M: Clone + 'static>(msg: M) -> Node<M> {
    a![C!["button", "is-light"], ev(Ev::Click, |_| msg), "Log out"]
}
