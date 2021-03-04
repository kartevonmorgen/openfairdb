use crate::{api, components::navbar};
use difference::{Changeset, Difference};
use ofdb_boundary::{ClearanceForPlace, PendingClearanceForPlace, ResultCount};
use ofdb_entities::{place::PlaceHistory, place::PlaceRevision};
use ofdb_seed::Api;
use seed::{prelude::*, *};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Mdl {
    token: String,
    place_clearances: HashMap<String, api::PlaceClearance>,
    expanded: HashMap<String, bool>,
    navbar: navbar::Mdl,
}

#[derive(Clone)]
pub enum Msg {
    GetPendingClearances,
    GotPendingClearances(Vec<PendingClearanceForPlace>),
    GotPlaceHistory(PlaceHistory),
    Toggle(String),
    Accept(String, u64),
    ClearanceResult(Result<Vec<String>, ClearanceError>),
    ConsoleLog(String),
    Navbar(navbar::Msg),
}

#[derive(Clone, Debug)]
pub enum ClearanceError {
    Fetch,
    Incomplete,
}

pub fn init(orders: &mut impl Orders<Msg>) -> Option<Mdl> {
    SessionStorage::get(crate::TOKEN_KEY)
        .map_err(|err| {
            log!("No token found", err);
        })
        .ok()
        .map(|token| {
            orders.send_msg(Msg::GetPendingClearances);
            Mdl {
                token,
                place_clearances: HashMap::new(),
                expanded: HashMap::new(),
                navbar: navbar::Mdl {
                    login_status: navbar::LoginStatus::LoggedIn,
                    menu_is_active: false,
                },
            }
        })
}

pub fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::GetPendingClearances => {
            orders.perform_cmd(get_pending_clearances(mdl.token.clone()));
        }
        Msg::GotPendingClearances(pending) => {
            for p in pending {
                let id = p.place_id.clone();
                if let Some(pc) = mdl.place_clearances.get_mut(&p.place_id) {
                    pc.pending = p;
                } else {
                    mdl.place_clearances.insert(
                        id.clone(),
                        api::PlaceClearance {
                            pending: p,
                            history: None,
                        },
                    );
                }
                orders.perform_cmd(get_place_history(mdl.token.clone(), id));
            }
        }
        Msg::GotPlaceHistory(ph) => {
            if let Some(pc) = mdl.place_clearances.get_mut(ph.place.id.as_str()) {
                pc.history = Some(ph);
            }
        }
        Msg::Toggle(id) => {
            mdl.expanded
                .entry(id)
                .and_modify(|e| *e = !*e)
                .or_insert(true);
        }
        Msg::Accept(id, rev_nr) => {
            let c = ClearanceForPlace {
                place_id: id,
                cleared_revision: Some(rev_nr),
            };
            let clearances = vec![c];
            let token = mdl.token.to_owned();
            orders.perform_cmd(places_clearance(token, clearances));
        }
        Msg::ClearanceResult(Ok(ids)) => {
            for id in ids {
                mdl.place_clearances.remove(&id);
            }
            orders.perform_cmd(get_pending_clearances(mdl.token.clone()));
        }
        Msg::ClearanceResult(Err(err)) => {
            // TODO: handle error, e.g. show error message to the user
            error!(err);
        }
        Msg::ConsoleLog(str) => log!(str),
        Msg::Navbar(msg) => match msg {
            navbar::Msg::Logout => {
                if let Err(err) = SessionStorage::remove(crate::TOKEN_KEY) {
                    error!(err);
                }
                Url::reload();
            }
            _ => {
                navbar::update(msg, &mut mdl.navbar, &mut orders.proxy(Msg::Navbar));
            }
        },
    }
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    let li = mdl.place_clearances.iter().map(|(_, pc)| {
        let id = &pc.pending.place_id;
        let expanded = *mdl.expanded.get(id).unwrap_or(&false);
        let toggle_msg = Msg::Toggle(id.clone());

        li![
            C!["panel-block"],
            div![
                div![
                    button![
                        C!["button", "is-small"],
                        ev(Ev::Click, |_| toggle_msg),
                        span![
                            C!["icon", "is-small"],
                            i![if expanded {
                                C!["fa", "fa-chevron-down"]
                            } else {
                                C!["fa", "fa-chevron-right"]
                            }],
                        ]
                    ],
                    " ",
                    pc.overview_title(),
                ],
                if expanded {
                    if let Some(curr_rev) = pc.current_rev() {
                        div![details_table(
                            &pc.pending.place_id,
                            pc.last_cleared_rev_nr(),
                            pc.last_cleared_rev(),
                            curr_rev,
                        )]
                    } else {
                        p!["Loading current revision ..."]
                    }
                } else {
                    empty![]
                }
            ]
        ]
    });
    div![
        navbar::view(&mdl.navbar).map_msg(Msg::Navbar),
        main![div![
            C!["container"],
            div![
                C!["section"],
                h2![C!["title"], "Overview"],
                if li.clone().count() == 0 {
                    p!["There is nothing to clear :)"]
                } else {
                    ul![C!["panel"], li]
                }
            ]
        ]]
    ]
}

fn details_table(
    id: &str,
    last_cleared_rev_nr: Option<u64>,
    lastrev: Option<&PlaceRevision>,
    currrev: &PlaceRevision,
) -> Node<Msg> {
    let accept_msg = Msg::Accept(id.to_string(), currrev.revision.into());
    let title_cs = changeset(lastrev, currrev, |r| r.title.clone());
    let desc_cs = changeset(lastrev, currrev, |r| r.description.clone());
    let location_cs = location_cs(lastrev, currrev);
    let contact_cs = contact_cs(lastrev, currrev);
    let opening_cs = opening_cs(lastrev, currrev);
    let links_cs = links_cs(lastrev, currrev);
    let tags_cs = changeset_split(lastrev, currrev, "\n", |r| r.tags.join("<br>\n"));
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

    table![
        C!["details-table"],
        col![C!["col-head"]],
        col![C!["col-last"]],
        col![C!["col-curr"]],
        tr![
            th![],
            th!["Last checked ", last_rev],
            th![
                a![
                    attrs! {
                        At::Target => "_blank",
                        At::Rel => "noopener noreferrer",
                        At::Href => href,
                    },
                    "Current ",
                    format!("(rev {})", u64::from(currrev.revision))
                ],
                " ",
                button![
                    C!["button", "is-primary"],
                    "Accept",
                    ev(Ev::Click, |_| accept_msg)
                ],
            ]
        ],
        table_row_always("Title", &title_cs),
        table_row_always("Description", &desc_cs),
        table_row_always("Location", &location_cs),
        table_row("Contact", &contact_cs),
        table_row("Opening hours", &opening_cs),
        table_row("Links", &links_cs),
        table_row("Tags", &tags_cs),
    ]
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
            street = addr.clone().map(|a| a.street).flatten().unwrap_or_default(),
            zip = addr.clone().map(|a| a.zip).flatten().unwrap_or_default(),
            city = addr.clone().map(|a| a.city).flatten().unwrap_or_default(),
            country = addr
                .clone()
                .map(|a| a.country)
                .flatten()
                .unwrap_or_default(),
            state = addr.clone().map(|a| a.state).flatten().unwrap_or_default(),
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
                .map(|c| c.email.map(String::from))
                .flatten()
                .unwrap_or_default(),
            phone = c
                .clone()
                .map(|c| c.phone.map(String::from))
                .flatten()
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
                .map(|l| l.homepage)
                .flatten()
                .map(|h| h.into_string())
                .unwrap_or_default(),
            image = l
                .clone()
                .map(|l| l.image)
                .flatten()
                .map(|h| h.into_string())
                .unwrap_or_default(),
            imagehref = l
                .clone()
                .map(|l| l.image_href)
                .flatten()
                .map(|h| h.into_string())
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

fn table_row_always<Ms>(title: &str, cs: &Changeset) -> Node<Ms> {
    tr![td![title], td![diffy_last(cs)], td![diffy_current(cs)],]
}

fn table_row<Ms>(title: &str, cs: &Changeset) -> Node<Ms> {
    if cs.distance == 0 {
        empty![]
    } else {
        table_row_always(title, cs)
    }
}

fn diffy_current<Ms>(cs: &Changeset) -> Node<Ms> {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => span![raw![s]],
        Difference::Add(s) => span![C!["diffadd"], raw![s]],
        _ => empty![],
    });
    span![csm]
}

fn diffy_last<Ms>(cs: &Changeset) -> Node<Ms> {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => span![raw![s]],
        Difference::Rem(s) => span![C!["diffrem"], raw![s]],
        _ => empty![],
    });
    span![csm]
}

async fn get_pending_clearances(api_token: String) -> Option<Msg> {
    let api = Api::new(api::API_ROOT.into());
    match api.get_places_clearance_with_api_token(&api_token).await {
        Ok(pending) => Some(Msg::GotPendingClearances(pending)),
        Err(err) => {
            error!(err);
            if let FetchError::StatusError(Status { code, .. }) = err {
                if code == 401 {
                    let url = Url::new()
                        .set_path(&[crate::PAGE_URL])
                        .set_hash_path(&[crate::HASH_PATH_LOGIN, crate::HASH_PATH_INVALID]);
                    url.go_and_load();
                }
            }
            None
        }
    }
}

async fn get_place_history(api_token: String, id: String) -> Option<Msg> {
    let api = Api::new(api::API_ROOT.into());
    match api.get_place_history_with_api_token(&api_token, &id).await {
        Ok(ph) => {
            let ph = PlaceHistory::from(ph);
            Some(Msg::GotPlaceHistory(ph))
        }
        Err(err) => {
            error!(err);
            return None;
        }
    }
}

async fn places_clearance(token: String, clearances: Vec<ClearanceForPlace>) -> Msg {
    let api = Api::new(api::API_ROOT.into());
    let cnt = clearances.len();
    let ids = clearances
        .iter()
        .map(|c| &c.place_id)
        .map(|id| id.to_string())
        .collect();
    match api
        .post_places_clearance_with_api_token(&token, clearances)
        .await
    {
        Ok(ResultCount { count }) => {
            if count as usize == cnt {
                Msg::ClearanceResult(Ok(ids))
            } else {
                Msg::ClearanceResult(Err(ClearanceError::Incomplete))
            }
        }
        Err(err) => {
            error!(err);
            Msg::ClearanceResult(Err(ClearanceError::Fetch))
        }
    }
}
