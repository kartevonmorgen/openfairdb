use std::collections::HashMap;

use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_router::*;
use ofdb_entities::place::PlaceHistory;
use ofdb_frontend_api::Api;
use wasm_bindgen::JsCast;

mod api;
mod components;
mod page;

use components::navbar::Navbar;
use page::{index::Index, login::Login, Page};

const TOKEN_KEY: &str = "org-token";
const TITLE: &str = "Clearance Center";

#[component]
fn App(cx: Scope) -> impl IntoView {
    // -- signals -- //

    let (token, set_token) = create_signal(cx, Option::<String>::None);
    let logged_in = Signal::derive(cx, move || token.get().is_some());
    let invalid_token = create_rw_signal(cx, false);
    let place_clearances = create_rw_signal(cx, HashMap::<String, api::PlaceClearance>::new());

    // -- actions -- //

    let get_place_history = create_action(cx, move |(token, id): &(String, String)| {
        let api = Api::new(api::API_ROOT.into());
        let token = token.clone();
        let id = id.clone();
        async move {
            match api.get_place_history_with_api_token(&token, &id).await {
                Ok(ph) => {
                    let ph = match PlaceHistory::try_from(ph) {
                        Ok(ph) => ph,
                        Err(err) => {
                            log::warn!("Unable to use place history: {err}");
                            return;
                        }
                    };
                    place_clearances.update(|place_clearances| {
                        if let Some(pc) = place_clearances.get_mut(ph.place.id.as_str()) {
                            pc.history = Some(ph);
                        }
                    });
                }
                Err(err) => {
                    log::error!("{err}");
                }
            }
        }
    });

    let fetch_pending_clearances = create_action(cx, move |_: &()| async move {
        match token.get() {
            Some(token) => {
                let api = Api::new(api::API_ROOT.into());
                match api.get_places_clearance_with_api_token(&token).await {
                    Ok(pending) => {
                        invalid_token.update(|v| *v = false);
                        place_clearances.update(|place_clearances| {
                            for p in pending {
                                let id = p.place_id.clone();
                                if let Some(pc) = place_clearances.get_mut(&p.place_id) {
                                    pc.pending = p;
                                } else {
                                    place_clearances.insert(
                                        id.clone(),
                                        api::PlaceClearance {
                                            pending: p,
                                            history: None,
                                        },
                                    );
                                }
                                let token: String = token.clone();
                                let id: String = id.clone();
                                get_place_history.dispatch((token, id));
                            }
                        });
                    }
                    Err(err) => {
                        log::error!("{err}");
                        if let ofdb_frontend_api::Error::Api(ofdb_boundary::Error {
                            http_status,
                            ..
                        }) = err
                        {
                            if http_status == 401 {
                                set_token.update(|v| *v = None);
                                invalid_token.update(|v| *v = true);
                            }
                        }
                    }
                }
            }
            None => {
                log::error!("Unable to fetch pending clearances: not logged in")
            }
        }
    });

    // -- init -- //

    if let Ok(token) = SessionStorage::get(TOKEN_KEY) {
        set_token.update(|t| *t = Some(token));
        fetch_pending_clearances.dispatch(());
    }

    // -- effects -- //

    view! { cx,
        <Router>
            <Navbar logged_in/>
            <main>
                <Routes>
                    <Route
                        path=Page::Home.path()
                        view=move |cx| {
                            if let Some(token) = token.get() {
                                view! { cx,
                                    <Index token place_clearances fetch_pending_clearances/>
                                }
                                    .into_view(cx)
                            } else {
                                let navigate = leptos_router::use_navigate(cx);
                                request_animation_frame(move || {
                                    _ = navigate(Page::Login.path(), Default::default());
                                });
                                view! {
                                    cx,
                                }
                                    .into_view(cx)
                            }
                        }
                    />
                    <Route
                        path=Page::Login.path()
                        view=move |cx| view! { cx, <Login invalid_token=invalid_token.into()/> }
                    />
                    <Route
                        path=Page::Logout.path()
                        view=move |cx| {
                            SessionStorage::delete(TOKEN_KEY);
                            set_token.update(|x| *x = None);
                            let navigate = leptos_router::use_navigate(cx);
                            request_animation_frame(move || {
                                _ = navigate(Page::Login.path(), Default::default());
                            });
                            view! {
                                cx,
                            }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}

pub fn run() {
    let container = document()
        .get_element_by_id("app")
        .expect("container element");
    mount_to(container.unchecked_into(), |cx| view! { cx, <App/> });
}
