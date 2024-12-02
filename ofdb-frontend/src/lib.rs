use anyhow::anyhow;
use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::JsFuture;

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

#[allow(clippy::too_many_lines)] // TODO
#[component]
#[must_use]
pub fn App() -> impl IntoView {
    // -- signals -- //

    let user_api = RwSignal::new(None::<api::UserApi>);
    let user_info = RwSignal::new(None::<User>);
    let logged_in = Signal::derive(move || user_api.get().is_some());
    let (bbox, _) = create_signal(DEFAULT_BBOX);

    // -- signal modifiers -- //

    let clear_user_data = move || {
        user_api.update(|a| *a = None);
        user_info.update(|i| *i = None);
    };

    // -- actions -- //

    let fetch_user_info = Action::new(move |()| async move {
        match user_api.get_untracked() {
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
                log::error!("Unable to fetch user info: not logged in");
            }
        }
    });

    let logout = Action::new(move |()| async move {
        match user_api.get_untracked() {
            Some(api) => match api.logout().await {
                Ok(()) => {
                    clear_user_data();
                }
                Err(err) => {
                    log::error!("Unable to logout: {err}");
                }
            },
            None => {
                log::error!("Unable to logout user: not logged in");
            }
        }
    });

    let copy_token = Action::new(move |()| async move {
        let Some(api) = user_api.get_untracked() else {
            log::warn!("Expect available user API");
            return;
        };
        log::debug!("Copy API token to clipboard");
        if let Err(err) = copy_to_clipboard(&api.token().token).await {
            log::warn!("Unable to copy token: {err}");
        }
    });

    // -- callbacks -- //

    let on_logout = move || {
        logout.dispatch(());
    };

    let on_copy_token = move || {
        copy_token.dispatch(());
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

    Effect::new(move |_| {
        log::debug!("API authorization state changed");
        if let Some(api) = user_api.get() {
            log::debug!("API is now authorized: save token in LocalStorage");
            LocalStorage::set(API_TOKEN_STORAGE_KEY, api.token()).expect("LocalStorage::set");
        } else {
            log::debug!("API is no longer authorized: delete token from LocalStorage");
            LocalStorage::delete(API_TOKEN_STORAGE_KEY);
        }
    });

    view! {
      <Router>
        <NavBar user = user_info.into() on_logout on_copy_token />
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
                      navigate(Page::Dashboard.path(), NavigateOptions::default());
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
              view=move|| view! {
                <Entry
                  public_api
                  user_api = user_api.into()
                />
              }
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

async fn copy_to_clipboard(text: &str) -> anyhow::Result<()> {
    let clipboard = window().navigator().clipboard();
    let promise = clipboard.write_text(text);
    JsFuture::from(promise).await.map_err(|err| {
        anyhow!(err
            .as_string()
            .unwrap_or_else(|| "unknown JS error".to_string()))
    })?;
    Ok(())
}
