use crate::api;
use difference::{Changeset, Difference};
use ofdb_boundary::ClearanceForPlace;
use ofdb_entities::{place::PlaceHistory, place::PlaceRevision};
use ofdb_seed::Api;
use seed::{prelude::*, *};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Mdl {
    token: String,
    place_clearances: Vec<api::PlaceClearance>,
    expanded: HashMap<String, bool>,
}

#[derive(Clone)]
pub enum Msg {
    GetPendingClearancesFull,
    GotPendingClearancesFull(Vec<api::PlaceClearance>),
    Toggle(String),
    Accept(String, u64),
    Logout,
    ConsoleLog(String),
}

pub fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::GetPendingClearancesFull => {
            orders.perform_cmd(get_pending_clearances_full(mdl.token.clone()));
        }
        Msg::GotPendingClearancesFull(pcs) => {
            mdl.place_clearances = pcs;
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
            orders.perform_cmd(async move {
                let api = Api::new(api::API_ROOT.into());
                let _res = api
                    .post_places_clearance_with_api_token(&token, clearances)
                    .await;
                Url::reload();
            });
        }
        Msg::Logout => {
            if let Err(err) = SessionStorage::remove(crate::TOKEN_KEY) {
                error!(err);
            }
            Url::reload();
        }
        Msg::ConsoleLog(str) => log!(str),
    }
}

pub fn init(orders: &mut impl Orders<Msg>) -> Option<Mdl> {
    SessionStorage::get(crate::TOKEN_KEY)
        .map_err(|err| {
            log!("No token found", err);
        })
        .ok()
        .map(|token| {
            orders.send_msg(Msg::GetPendingClearancesFull);
            Mdl {
                token,
                place_clearances: Vec::new(),
                expanded: HashMap::new(),
            }
        })
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    let li = mdl.place_clearances.iter().map(|pc| {
        let id = &pc.pending.place_id;
        let lastrev = pc.last_cleared_rev();
        let currrev = pc.current_rev();
        let title_cs = changeset(lastrev, currrev, |r| r.title.clone());
        let desc_cs = changeset(lastrev, currrev, |r| r.description.clone());
        let location_cs = changeset(lastrev, currrev, |r| {
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
        });
        let contact_cs = changeset(lastrev, currrev, |r| {
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
        });
        let opening_cs = changeset(lastrev, currrev, |r| {
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
        });
        let links_cs = changeset(lastrev, currrev, |r| {
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
        });
        let tags_cs = changeset_split(lastrev, currrev, "\n", |r| r.tags.join("<br>\n"));
        let expanded = *mdl.expanded.get(id).unwrap_or(&false);
        let toggle_msg = Msg::Toggle(id.clone());
        let accept_msg = Msg::Accept(id.clone(), pc.current_rev_nr());

        li![
            pc.overview_title(),
            " ",
            button![
                ev(Ev::Click, |_| toggle_msg),
                i![
                    style! {
                        St::Width => px(20),
                        St::TextAlign => "center",
                    },
                    if expanded {
                        C!["fa", "fa-chevron-down"]
                    } else {
                        C!["fa", "fa-chevron-right"]
                    }
                ],
            ],
            if expanded {
                let last_rev = if let Some(nr) = pc.last_cleared_rev_nr() {
                    format!("(rev {})", nr)
                } else {
                    String::new()
                };
                let center = currrev.location.pos;
                let href = format!(
                    "https://kartevonmorgen.org/#/?entry={}&center={},{}&zoom=15.00",
                    id,
                    center.lat(),
                    center.lng()
                );
                div![table![
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
                                format!("(rev {})", pc.current_rev_nr())
                            ],
                            " ",
                            button!["Accept", ev(Ev::Click, |_| accept_msg)],
                        ]
                    ],
                    table_row_always("Title", &title_cs),
                    table_row_always("Description", &desc_cs),
                    table_row_always("Location", &location_cs),
                    table_row("Contact", &contact_cs),
                    table_row("Opening hours", &opening_cs),
                    table_row("Links", &links_cs),
                    table_row("Tags", &tags_cs),
                ]]
            } else {
                empty![]
            }
        ]
    });
    div![
        div![
            style! {
                St::Float => "right",
            },
            button![ev(Ev::Click, |_| Msg::Logout), "Logout",],
        ],
        h1![crate::TITLE],
        h2!["Overview"],
        if li.clone().count() == 0 {
            p!["There is nothing to clear :)"]
        } else {
            ul![li]
        }
    ]
}

pub fn changeset<F>(last: Option<&PlaceRevision>, curr: &PlaceRevision, f: F) -> Changeset
where
    F: Fn(&PlaceRevision) -> String,
{
    changeset_split(last, curr, "", f)
}

pub fn changeset_split<F>(
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

pub fn table_row_always<Ms>(title: &str, cs: &Changeset) -> Node<Ms> {
    tr![td![title], td![diffy_last(cs)], td![diffy_current(cs)],]
}

pub fn table_row<Ms>(title: &str, cs: &Changeset) -> Node<Ms> {
    if cs.distance == 0 {
        empty![]
    } else {
        table_row_always(title, cs)
    }
}

pub fn diffy_current<Ms>(cs: &Changeset) -> Node<Ms> {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => span![raw![s]],
        Difference::Add(s) => span![C!["diffadd"], raw![s]],
        _ => empty![],
    });
    span![csm]
}

pub fn diffy_last<Ms>(cs: &Changeset) -> Node<Ms> {
    let csm = cs.diffs.iter().map(|d| match d {
        Difference::Same(s) => span![raw![s]],
        Difference::Rem(s) => span![C!["diffrem"], raw![s]],
        _ => empty![],
    });
    span![csm]
}

pub async fn get_pending_clearances_full(api_token: String) -> Option<Msg> {
    let api = Api::new(api::API_ROOT.into());
    match api.get_places_clearance_with_api_token(&api_token).await {
        Ok(pend) => {
            let mut rezz = Vec::new();
            for i in pend {
                match api
                    .get_place_history_with_api_token(&api_token, &i.place_id)
                    .await
                {
                    Ok(ph) => {
                        let ph = PlaceHistory::from(ph);
                        rezz.push(api::PlaceClearance {
                            pending: i,
                            history: ph,
                            expanded: false,
                        });
                    }
                    Err(err) => {
                        error!(err);
                        return None;
                    }
                }
            }
            Some(Msg::GotPendingClearancesFull(rezz))
        }
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
