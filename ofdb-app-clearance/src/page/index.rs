use std::collections::{HashMap, HashSet};

use leptos::*;

use difference::{Changeset, Difference};
use ofdb_boundary::{ClearanceForPlace, ResultCount};
use ofdb_entities::{email::EmailAddress, place::PlaceRevision};
use ofdb_frontend_api::ClearanceApi;

use crate::api;

#[component]
pub fn Index(
    token: String,
    place_clearances: RwSignal<HashMap<String, api::PlaceClearance>>,
    fetch_pending_clearances: Action<(), ()>,
) -> impl IntoView {
    let expanded = create_rw_signal(HashSet::<String>::new());
    let selected = create_rw_signal(HashSet::<String>::new());

    let handle_clearance_result = move |result| {
        match result {
            Ok(ids) => {
                for id in ids {
                    place_clearances.update(|x| {
                        x.remove(&id);
                    });
                    selected.update(|x| {
                        x.remove(&id);
                    });
                    expanded.update(|x| {
                        x.remove(&id);
                    });
                }
            }
            Err(err) => {
                // TODO: handle error, e.g. show error message to the user
                log::error!("{err:?}");
            }
        }
        fetch_pending_clearances.dispatch(());
    };

    let accept = {
        let token = token.clone();
        create_action(move |(id, rev_nr): &(String, u64)| {
            let c = ClearanceForPlace {
                place_id: id.clone(),
                cleared_revision: Some(*rev_nr),
            };
            let clearances = vec![c];
            let token = token.clone();
            async move {
                let result = places_clearance(token, clearances).await;
                handle_clearance_result(result);
            }
        })
    };

    let accept_all_selected = {
        let token = token.clone();
        create_action(move |_: &()| {
            let place_clearances = place_clearances.get();
            let clearances = selected
                .get()
                .iter()
                .filter_map(|id| place_clearances.get(id).map(|pc| (id, pc)))
                .filter_map(|(id, pc)| {
                    pc.current_rev()
                        .map(|rev| rev.revision)
                        .map(u64::from)
                        .map(Option::Some)
                        .map(|rev| (id.to_string(), rev))
                })
                .map(|(place_id, cleared_revision)| ClearanceForPlace {
                    place_id,
                    cleared_revision,
                })
                .collect();
            let token = token.clone();
            async move {
                let result = places_clearance(token, clearances).await;
                handle_clearance_result(result);
            }
        })
    };

    view! {
        <div>
            <main>
                <div class="container">
                    <div class="section">
                        {move || {
                            if place_clearances.get().is_empty() {
                                view! { <p>"There is nothing to clear :)"</p> }.into_view()
                            } else {
                                view! {
                                    <div class="panel">
                                        <p class="panel-heading">
                                            "Pending Clearances"
                                            <span class="subtitle is-5">
                                                " (" {place_clearances.get().len()} ")"
                                            </span>
                                        </p>
                                        <div class="panel-block">
                                            <PanelActions place_clearances expanded selected/>
                                        </div>
                                        <ul>
                                            <For
                                                each=move || place_clearances.get()
                                                key=|(id, _)| id.clone()
                                                view=move |(_, place_clearance)| {
                                                    view! {
                                                        <PanelBlock place_clearance expanded selected accept/>
                                                    }
                                                }
                                            />
                                        </ul>
                                        <div class="panel-block">
                                            <button
                                                class="button is-danger is-outlined is-fullwidth"
                                                disabled=move || selected.get().is_empty()
                                                on:click=move |_| accept_all_selected.dispatch(())
                                            >
                                                {format!("Accept all ({}) selected", selected.get().len())}
                                            </button>
                                        </div>
                                    </div>
                                }
                                    .into_view()
                            }
                        }}
                    </div>
                </div>
            </main>
        </div>
    }
}

#[component]
fn PanelBlock(
    place_clearance: api::PlaceClearance,
    expanded: RwSignal<HashSet<String>>,
    selected: RwSignal<HashSet<String>>,
    accept: Action<(String, u64), ()>,
) -> impl IntoView {
    let id = place_clearance.pending.place_id.clone();
    let is_expanded = {
        let id = id.clone();
        Signal::derive(move || expanded.get().contains(&id))
    };
    let is_selected = {
        let id = id.clone();
        Signal::derive(move || selected.get().contains(&id))
    };

    view! {
        <li class="panel-block">
            <div>
                <div class="level">
                    <div class="level-left">
                        <div class="level-item">
                            <div class="field is-grouped">
                                <p class="control">
                                    <label class="checkbox">
                                        <input
                                            type="checkbox"
                                            checked=move || is_selected.get()
                                            on:click={
                                                let id = id.clone();
                                                move |_| {
                                                    selected
                                                        .update(|selected| {
                                                            if selected.contains(&id) {
                                                                selected.remove(&id);
                                                            } else {
                                                                selected.insert(id.clone());
                                                            }
                                                        })
                                                }
                                            }
                                        />
                                    </label>
                                </p>
                                <p class="control">
                                    <button
                                        class="button is-small"
                                        on:click=move |_| {
                                            expanded
                                                .update(|expanded| {
                                                    if expanded.contains(&id) {
                                                        expanded.remove(&id);
                                                    } else {
                                                        expanded.insert(id.clone());
                                                    }
                                                })
                                        }
                                    >
                                        <span class="icon is-small">
                                            <i class=move || {
                                                if is_expanded.get() {
                                                    "fa fa-chevron-down"
                                                } else {
                                                    "fa fa-chevron-right"
                                                }
                                            }></i>

                                        // #[derive(Debug)]
                                        // pub struct Mdl {
                                        //     token: String,
                                        //     place_clearances: HashMap<String, api::PlaceClearance>,
                                        //     expanded: HashSet<String>,
                                        //     selected: HashSet<String>,
                                        //     navbar: navbar::Mdl,
                                        // }
                                        //
                                        // #[derive(Clone)]
                                        // pub enum Msg {
                                        //     GetPendingClearances,
                                        //     GotPendingClearances(Vec<PendingClearanceForPlace>),
                                        //     GotPlaceHistory(PlaceHistory),
                                        //     ToggleExpand(String),
                                        //     ToggleSelect(String),
                                        //     Accept(String, u64),
                                        //     AcceptAllSelected,
                                        //     ClearanceResult(Result<Vec<String>, ClearanceError>),
                                        //     ConsoleLog(String),
                                        //     Navbar(navbar::Msg),
                                        //     ExpandAll,
                                        //     SelectAll,
                                        //     DeselectAll,
                                        //     CollapseAll,
                                        // }

                                        // pub fn init(orders: &mut impl Orders<Msg>) -> Option<Mdl> {
                                        //     SessionStorage::get(crate::TOKEN_KEY)
                                        //         .map_err(|err| {
                                        //             log::debug!("No token found {err}");
                                        //         })
                                        //         .ok()
                                        //         .map(|token| {
                                        //             orders.send_msg(Msg::GetPendingClearances);
                                        //             Mdl {
                                        //                 token,
                                        //                 place_clearances: HashMap::new(),
                                        //                 expanded: HashSet::new(),
                                        //                 selected: HashSet::new(),
                                        //                 navbar: navbar::Mdl {
                                        //                     login_status: navbar::LoginStatus::LoggedIn,
                                        //                     menu_is_active: false,
                                        //                 },
                                        //             }
                                        //         })
                                        // }
                                        //
                                        // pub fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
                                        //     match msg {
                                        //         Msg::GetPendingClearances => {
                                        //             orders.perform_cmd(get_pending_clearances(mdl.token.clone()));
                                        //         }
                                        //         Msg::GotPendingClearances(pending) => {
                                        //             for p in pending {
                                        //                 let id = p.place_id.clone();
                                        //                 if let Some(pc) = mdl.place_clearances.get_mut(&p.place_id) {
                                        //                     pc.pending = p;
                                        //                 } else {
                                        //                     mdl.place_clearances.insert(
                                        //                         id.clone(),
                                        //                         api::PlaceClearance {
                                        //                             pending: p,
                                        //                             history: None,
                                        //                         },
                                        //                     );
                                        //                 }
                                        //                 orders.perform_cmd(get_place_history(mdl.token.clone(), id));
                                        //             }
                                        //         }
                                        //         Msg::GotPlaceHistory(ph) => {
                                        //             if let Some(pc) = mdl.place_clearances.get_mut(ph.place.id.as_str()) {
                                        //                 pc.history = Some(ph);
                                        //             }
                                        //         }
                                        //         Msg::ToggleExpand(id) => {
                                        //             if mdl.expanded.contains(&id) {
                                        //                 mdl.expanded.remove(&id);
                                        //             } else {
                                        //                 mdl.expanded.insert(id);
                                        //             }
                                        //         }
                                        //         Msg::ToggleSelect(id) => {
                                        //             if mdl.selected.contains(&id) {
                                        //                 mdl.selected.remove(&id);
                                        //             } else {
                                        //                 mdl.selected.insert(id);
                                        //             }
                                        //         }
                                        //         Msg::ExpandAll => {
                                        //             mdl.expanded = mdl.place_clearances.keys().cloned().collect();
                                        //         }
                                        //         Msg::SelectAll => {
                                        //             mdl.selected = mdl.place_clearances.keys().cloned().collect();
                                        //         }
                                        //         Msg::DeselectAll => {
                                        //             mdl.selected.clear();
                                        //         }
                                        //         Msg::CollapseAll => {
                                        //             mdl.expanded.clear();
                                        //         }
                                        //         Msg::Accept(id, rev_nr) => {
                                        //         }
                                        //         Msg::ClearanceResult(Ok(ids)) => {
                                        //             for id in ids {
                                        //                 mdl.place_clearances.remove(&id);
                                        //                 mdl.selected.remove(&id);
                                        //                 mdl.expanded.remove(&id);
                                        //             }
                                        //             orders.perform_cmd(get_pending_clearances(mdl.token.clone()));
                                        //         }
                                        //         Msg::ClearanceResult(Err(err)) => {
                                        //
                                        //             log::error!("{err:?}");
                                        //         }
                                        //         Msg::ConsoleLog(msg) => log::debug!("{msg}"),
                                        //         Msg::Navbar(msg) => match msg {
                                        //             navbar::Msg::Logout => {
                                        //                 SessionStorage::delete(crate::TOKEN_KEY);
                                        //                 Url::reload();
                                        //             }
                                        //             _ => {
                                        //                 navbar::update(msg, &mut mdl.navbar, &mut orders.proxy(Msg::Navbar));
                                        //             }
                                        //         },
                                        //     }
                                        // }

                                        // TODO: handle error, e.g. show error message to the user

                                        </span>
                                    </button>
                                </p>
                            </div>
                        </div>
                        <div class="level-item">{place_clearance.overview_title().to_string()}</div>
                    </div>
                </div>
                {move || {
                    is_expanded
                        .get()
                        .then(|| {
                            if let Some(curr_rev) = place_clearance.current_rev() {
                                view! {
                                    <div>
                                        <DetailsTable
                                            id=place_clearance.pending.place_id.clone()
                                            last_cleared_rev_nr=place_clearance.last_cleared_rev_nr()
                                            lastrev=place_clearance.last_cleared_rev().cloned()
                                            currrev=curr_rev.clone()
                                            accept
                                        />
                                    </div>
                                }
                                    .into_view()
                            } else {
                                view! { <p>"Loading current revision ..."</p> }.into_view()
                            }
                        })
                }}
            </div>
        </li>
    }
}

#[component]
fn PanelActions(
    place_clearances: RwSignal<HashMap<String, api::PlaceClearance>>,
    expanded: RwSignal<HashSet<String>>,
    selected: RwSignal<HashSet<String>>,
) -> impl IntoView {
    let expand_all = move |_| {
        expanded.update(|expanded| {
            *expanded = place_clearances.get().keys().cloned().collect();
        });
    };
    let select_all = move |_| {
        selected.update(|selected| {
            *selected = place_clearances.get().keys().cloned().collect();
        });
    };
    let deselect_all = move |_| {
        selected.update(|x| x.clear());
    };
    let collapse_all = move |_| {
        expanded.update(|x| x.clear());
    };
    view! {
        <div class="field is-grouped">
            <p class="control">
                <button class="button" on:click=expand_all>
                    "expand all"
                </button>
            </p>
            <p class="control">
                <button class="button" on:click=collapse_all>
                    "collapse all"
                </button>
            </p>
            <p class="control">
                <button class="button" on:click=select_all>
                    "select all"
                </button>
            </p>
            <p class="control">
                <button class="button" on:click=deselect_all>
                    "deselect all"
                </button>
            </p>
        </div>
    }
}

#[component]
fn DetailsTable(
    id: String,
    last_cleared_rev_nr: Option<u64>,
    lastrev: Option<PlaceRevision>,
    currrev: PlaceRevision,
    accept: Action<(String, u64), ()>,
) -> impl IntoView {
    let title_cs = changeset(lastrev.as_ref(), &currrev, |r| r.title.clone());
    let desc_cs = changeset(lastrev.as_ref(), &currrev, |r| r.description.clone());
    let location_cs = location_cs(lastrev.as_ref(), &currrev);
    let contact_cs = contact_cs(lastrev.as_ref(), &currrev);
    let opening_cs = opening_cs(lastrev.as_ref(), &currrev);
    let links_cs = links_cs(lastrev.as_ref(), &currrev);
    let tags_cs = changeset_split(lastrev.as_ref(), &currrev, "\n", |r| r.tags.join("<br>\n"));
    let center = currrev.location.pos;
    let href = format!(
        "https://kartevonmorgen.org/#/?entry={}&center={},{}&zoom=15.00",
        id,
        center.lat(),
        center.lng()
    );

    let last_rev = if let Some(nr) = last_cleared_rev_nr {
        format!("(rev {})", nr)
    } else {
        String::new()
    };

    view! {
        <table class="details-table">
            <col class="col-head"/>
            <col class="col-last"/>
            <col class="col-curr"/>
            <tr>
                <th></th>
                <th>"Last checked " {last_rev}</th>
                <th>
                    <a target="_blank" rel="noopener noreferrer" href=href>
                        "Current "
                        {format!("(rev {})", u64::from(currrev.revision))}
                    </a>
                    " "
                    <button
                        class="button is-primary"
                        on:click=move |_| accept.dispatch((id.clone(), currrev.revision.into()))
                    >
                        "Accept"
                    </button>
                </th>
            </tr>
            {table_row_always("Title", &title_cs)}
            {table_row_always("Description", &desc_cs)}
            {table_row_always("Location", &location_cs)}
            {table_row("Contact", &contact_cs)}
            {table_row("Opening hours", &opening_cs)}
            {table_row("Links", &links_cs)}
            {table_row("Tags", &tags_cs)}
        </table>
    }
}

fn location_cs(lastrev: Option<&PlaceRevision>, currrev: &PlaceRevision) -> Changeset {
    changeset(lastrev, currrev, |r| {
        let pos = r.location.pos;
        let addr = &r.location.address;
        format!(
            r#"
           Lat {lat}, Lon {lon}<br>
           {street}<br>
           {zip}<br>
           {city}<br>
           {country}<br>
           {state}
           "#,
            lat = pos.lat(),
            lon = pos.lng(),
            street = addr.clone().and_then(|a| a.street).unwrap_or_default(),
            zip = addr.clone().and_then(|a| a.zip).unwrap_or_default(),
            city = addr.clone().and_then(|a| a.city).unwrap_or_default(),
            country = addr.clone().and_then(|a| a.country).unwrap_or_default(),
            state = addr.clone().and_then(|a| a.state).unwrap_or_default(),
        )
    })
}

fn contact_cs(lastrev: Option<&PlaceRevision>, currrev: &PlaceRevision) -> Changeset {
    changeset(lastrev, currrev, |r| {
        let c = &r.contact;
        format!(
            r#"
        {email}<br>
        {phone}
        "#,
            email = c
                .clone()
                .and_then(|c| c.email.map(EmailAddress::into_string))
                .unwrap_or_default(),
            phone = c
                .clone()
                .and_then(|c| c.phone.map(String::from))
                .unwrap_or_default(),
        )
    })
}

fn opening_cs(lastrev: Option<&PlaceRevision>, currrev: &PlaceRevision) -> Changeset {
    changeset(lastrev, currrev, |r| {
        format!(
            r#"
        {hours}
        "#,
            hours = &r
                .opening_hours
                .clone()
                .map(String::from)
                .unwrap_or_default(),
        )
    })
}

fn links_cs(lastrev: Option<&PlaceRevision>, currrev: &PlaceRevision) -> Changeset {
    changeset(lastrev, currrev, |r| {
        let l = &r.links;
        format!(
            r#"
        {homepage}<br>
        {image}<br>
        {imagehref}
        "#,
            homepage = l
                .clone()
                .and_then(|l| l.homepage)
                .map(Into::<String>::into)
                .unwrap_or_default(),
            image = l
                .clone()
                .and_then(|l| l.image)
                .map(Into::<String>::into)
                .unwrap_or_default(),
            imagehref = l
                .clone()
                .and_then(|l| l.image_href)
                .map(Into::<String>::into)
                .unwrap_or_default(),
        )
    })
}

fn changeset<F>(last: Option<&PlaceRevision>, curr: &PlaceRevision, f: F) -> Changeset
where
    F: Fn(&PlaceRevision) -> String,
{
    changeset_split(last, curr, "", f)
}

fn changeset_split<F>(
    last: Option<&PlaceRevision>,
    curr: &PlaceRevision,
    split: &str,
    f: F,
) -> Changeset
where
    F: Fn(&PlaceRevision) -> String,
{
    let slast = last.map_or(String::from(""), &f);
    let scurr = f(curr);
    Changeset::new(&slast, &scurr, split)
}

fn table_row_always(title: &'static str, cs: &Changeset) -> impl IntoView {
    view! {
        <tr>
            <td>{title}</td>
            <td>{diffy_last(cs)}</td>
            <td>{diffy_current(cs)}</td>
        </tr>
    }
}

fn table_row(title: &'static str, cs: &Changeset) -> impl IntoView {
    if cs.distance == 0 {
        None
    } else {
        Some(table_row_always(title, cs))
    }
}

fn diffy_current(cs: &Changeset) -> impl IntoView {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => Some(view! { <span inner_html=s></span> }.into_view()),
        Difference::Add(s) => {
            Some(view! { <span class="diffadd" inner_html=s></span> }.into_view())
        }
        _ => None,
    });
    view! { <span>{csm.collect_view()}</span> }
}

fn diffy_last(cs: &Changeset) -> impl IntoView {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => Some(view! { <span inner_html=s></span> }.into_view()),
        Difference::Rem(s) => {
            Some(view! { <span class="diffrem" inner_html=s></span> }.into_view())
        }
        _ => None,
    });
    view! { <span>{csm.collect_view()}</span> }
}

#[derive(Clone, Debug)]
pub enum ClearanceError {
    Fetch,
    Incomplete,
}

async fn places_clearance(
    token: String,
    clearances: Vec<ClearanceForPlace>,
) -> Result<Vec<String>, ClearanceError> {
    let api = ClearanceApi::new(api::API_ROOT, token);
    let cnt = clearances.len();
    let ids = clearances
        .iter()
        .map(|c| &c.place_id)
        .map(|id| id.to_string())
        .collect();
    match api.update_place_clearances(clearances).await {
        Ok(ResultCount { count }) => {
            if count as usize == cnt {
                Ok(ids)
            } else {
                Err(ClearanceError::Incomplete)
            }
        }
        Err(err) => {
            log::error!("{err}");
            Err(ClearanceError::Fetch)
        }
    }
}
