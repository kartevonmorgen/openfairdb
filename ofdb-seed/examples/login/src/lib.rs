use ofdb_seed::{boundary, components::login, Api};
use seed::{prelude::*, *};

#[derive(Clone)]
struct Mdl {
    api: Api,
    user_email: Option<String>,
    page: Page,
}

#[derive(Clone)]
enum Page {
    Home,
    Login(login::Mdl),
}

impl Default for Mdl {
    fn default() -> Self {
        Self {
            api: Api::new("/v0".to_string()),
            user_email: None,
            page: Page::Login(Default::default()),
        }
    }
}

enum Msg {
    Login(login::Msg),
    LoginResult(fetch::Result<()>),
    Logout,
    LogoutResult(fetch::Result<()>),
    GetCurrentUser,
    CurrentUserResult(fetch::Result<boundary::User>),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Mdl {
    orders.send_msg(Msg::GetCurrentUser);
    Mdl::default()
}

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Login(msg) => match &mut mdl.page {
            Page::Login(login_mdl) => match msg {
                login::Msg::EmailChanged(email) => {
                    if email.is_empty() {
                        login_mdl.errors.email = Some("An email address is required".to_string());
                    }
                    login_mdl.email = email;
                }
                login::Msg::PasswordChanged(pw) => {
                    if pw.is_empty() {
                        login_mdl.errors.password = Some("A password is required".to_string());
                    }
                    login_mdl.password = pw;
                }
                login::Msg::Submit => {
                    login_mdl.is_submitting = true;
                    let email = login_mdl.email.clone();
                    let password = login_mdl.password.clone();
                    let api = mdl.api.clone();
                    orders.perform_cmd(async move {
                        Msg::LoginResult(
                            api.post_login(&boundary::Credentials { email, password })
                                .await,
                        )
                    });
                }
            },
            _ => unreachable!(),
        },
        Msg::LoginResult(res) => match res {
            Ok(()) => {
                mdl.page = Page::Home;
                orders.send_msg(Msg::GetCurrentUser);
            }
            Err(err) => {
                if let Page::Login(login_mdl) = &mut mdl.page {
                    login_mdl.is_submitting = false;
                    login_mdl.errors.form = Some("Login failed".to_string());
                }
                seed::error!(err);
            }
        },
        Msg::Logout => {
            let api = mdl.api.clone();
            orders.perform_cmd(async move { Msg::LogoutResult(api.post_logout().await) });
        }
        Msg::LogoutResult(res) => {
            match res {
                Ok(()) => {
                    mdl.page = Page::Login(Default::default());
                }
                Err(err) => {
                    // TODO: show err
                    seed::error!(err);
                }
            }
        }
        Msg::GetCurrentUser => {
            let api = mdl.api.clone();
            orders
                .perform_cmd(async move { Msg::CurrentUserResult(api.get_users_current().await) });
        }
        Msg::CurrentUserResult(res) => {
            match res {
                Ok(user) => {
                    mdl.user_email = Some(user.email);
                    mdl.page = Page::Home;
                }
                Err(err) => {
                    // TODO: show err
                    seed::error!(err);
                    mdl.page = Page::Login(Default::default());
                }
            }
        }
    }
}

fn view(mdl: &Mdl) -> Node<Msg> {
    let user = mdl.user_email.clone().unwrap_or(String::new());
    match &mdl.page {
        Page::Home => div![
            class!["page", "home"],
            h1!["OpenFairDB"],
            p!["You are logged in ", user],
            button![ev(Ev::Click, |_| Msg::Logout), "logout"]
        ],
        Page::Login(mdl) => div![
            class!["page", "login"],
            h1!["OpenFairDB Login"],
            login::view(&mdl).map_msg(Msg::Login)
        ],
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
