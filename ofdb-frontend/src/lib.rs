use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::*;

use ofdb_boundary::{MapBbox, MapPoint, User};
use ofdb_frontend_api as api;

mod pages;
use pages::*;

mod components;
use components::*;

const DEFAULT_API_URL: &str = "/api";
const API_TOKEN_STORAGE_KEY: &str = "api-token";
const DEFAULT_BBOX: MapBbox = MapBbox {
    sw: MapPoint {
        lat: 43.0,
        lng: -16.0,
    },
    ne: MapPoint {
        lat: 60.0,
        lng: 25.0,
    },
};

#[component]
pub fn App() -> impl IntoView {
    // -- signals -- //

    let user_api = create_rw_signal(None::<api::UserApi>);
    let user_info = create_rw_signal(None::<User>);
    let logged_in = Signal::derive(move || user_api.get().is_some());
    let (bbox, _) = create_signal(DEFAULT_BBOX);

    // -- signal modifiers -- //

    let clear_user_data = move || {
        user_api.update(|a| *a = None);
        user_info.update(|i| *i = None);
    };

    // -- actions -- //

    let fetch_user_info = create_action(move |_| async move {
        match user_api.get() {
            Some(api) => match api.user_info().await {
                Ok(info) => {
                    user_info.update(|i| *i = Some(info));
                }
                Err(err) => {
                    log::error!("Unable to fetch user info: {err}");
                    clear_user_data();
                }
            },
            None => {
                log::error!("Unable to fetch user info: not logged in")
            }
        }
    });

    let logout = create_action(move |_| async move {
        match user_api.get() {
            Some(api) => match api.logout().await {
                Ok(_) => {
                    clear_user_data();
                }
                Err(err) => {
                    log::error!("Unable to logout: {err}")
                }
            },
            None => {
                log::error!("Unable to logout user: not logged in")
            }
        }
    });

    // -- callbacks -- //

    let on_logout = move || {
        logout.dispatch(());
    };

    // -- init API -- //

    let public_api = api::PublicApi::new(DEFAULT_API_URL);

    if let Ok(token) = LocalStorage::get(API_TOKEN_STORAGE_KEY) {
        let api = api::UserApi::new(DEFAULT_API_URL, token);
        user_api.update(|a| *a = Some(api));
        fetch_user_info.dispatch(());
    }

    log::debug!("User is logged in: {}", logged_in.get());

    // -- effects -- //

    create_effect(move |_| {
        log::debug!("API authorization state changed");
        match user_api.get() {
            Some(api) => {
                log::debug!("API is now authorized: save token in LocalStorage");
                LocalStorage::set(API_TOKEN_STORAGE_KEY, api.token()).expect("LocalStorage::set");
            }
            None => {
                log::debug!("API is no longer authorized: delete token from LocalStorage");
                LocalStorage::delete(API_TOKEN_STORAGE_KEY);
            }
        }
    });

    view! {
      <Router>
        <NavBar user = user_info.into() on_logout />
        <main>
          <Routes>
            <Route
              path=Page::Home.path()
              view=move || view! { <Home public_api bbox /> }
            />
            <Route
              path=Page::Login.path()
              view=move || view! {
                <Login
                  public_api
                  on_success = move |api| {
                      log::info!("Successfully logged in");
                      user_api.update(|v| *v = Some(api));
                      let navigate = use_navigate();
                      navigate(Page::Dashboard.path(), Default::default());
                      fetch_user_info.dispatch(());
                  } />
              }
            />
            <Route
              path=Page::Register.path()
              view=move || view! { <Register public_api /> }
            />
            <Route
              path=Page::ResetPassword.path()
              view=move|| view! { <ResetPassword public_api /> }
            />
            <Route
              path=Page::Dashboard.path()
              view=move|| view! {
                <Dashboard
                  public_api
                  user_api = user_api.into()
                />
              }
            />
            <Route
              path=format!("{}/:id", Page::Entries.path())
              view=move|| view! { <Entry public_api /> }
            />
            <Route
              path=Page::Events.path()
              view=move|| view! { <Events public_api /> }
            />
            <Route
              path=format!("{}/:id", Page::Events.path())
              view=move|| view! {
                <Event
                  public_api
                  user_api = user_api.into()
                />
              }
            />
          </Routes>
        </main>
      </Router>
    }
}
