use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::*;
use wasm_bindgen::JsCast;

use ofdb_boundary::*;

mod api;

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
fn App(cx: Scope) -> impl IntoView {
    // -- signals -- //
    let authorized_api = create_rw_signal(cx, None::<api::AuthorizedApi>);
    let user_info = create_rw_signal(cx, None::<User>);
    let logged_in = Signal::derive(cx, move || authorized_api.get().is_some());
    let (bbox, _) = create_signal(cx, DEFAULT_BBOX);

    // -- actions -- //

    let fetch_user_info = create_action(cx, move |_| async move {
        match authorized_api.get() {
            Some(api) => match api.user_info().await {
                Ok(info) => {
                    user_info.update(|i| *i = Some(info));
                }
                Err(err) => {
                    log::error!("Unable to fetch user info: {err}")
                }
            },
            None => {
                log::error!("Unable to fetch user info: not logged in")
            }
        }
    });

    let logout = create_action(cx, move |_| async move {
        match authorized_api.get() {
            Some(api) => match api.logout().await {
                Ok(_) => {
                    authorized_api.update(|a| *a = None);
                    user_info.update(|i| *i = None);
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

    let unauthorized_api = api::UnauthorizedApi::new(DEFAULT_API_URL);
    if let Ok(token) = LocalStorage::get(API_TOKEN_STORAGE_KEY) {
        let api = api::AuthorizedApi::new(DEFAULT_API_URL, token);
        authorized_api.update(|a| *a = Some(api));
        fetch_user_info.dispatch(());
    }

    log::debug!("User is logged in: {}", logged_in.get());

    // -- effects -- //

    create_effect(cx, move |_| {
        log::debug!("API authorization state changed");
        match authorized_api.get() {
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

    view! { cx,
      <Router>
        <NavBar logged_in on_logout />
        <main>
          <Routes>
            <Route
              path=Page::Home.path()
              view=move |cx| view! { cx, <PlaceSearch api = unauthorized_api bbox /> } />
            <Route
              path=Page::Login.path()
              view=move |cx| view! { cx,
                <Login
                  api = unauthorized_api
                  on_success = move |api| {
                      log::info!("Successfully logged in");
                      authorized_api.update(|v| *v = Some(api));
                      let navigate = use_navigate(cx);
                      navigate(Page::Home.path(), Default::default()).expect("Home route");
                      fetch_user_info.dispatch(());
                  } />
              }
            />
            <Route
              path=Page::Register.path()
              view=move |cx| view! { cx,
                <Register api = unauthorized_api />
              }
            />
            <Route
              path=Page::ResetPassword.path()
              view=move|cx| view! { cx, <ResetPassword api = unauthorized_api /> }
            />
          </Routes>
        </main>
      </Router>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::info!("Start web application");
    let app_container = document()
        .get_element_by_id("app")
        .expect("app container element")
        .dyn_into()
        .expect("HtmlElement");
    mount_to(app_container, |cx| view! { cx, <App /> });
}
