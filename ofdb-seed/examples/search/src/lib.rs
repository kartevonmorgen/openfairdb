use ofdb_seed::{
    boundary::{MapBbox, MapPoint, SearchResponse},
    components::{search_bar, search_result_item},
    Api,
};
use seed::{prelude::*, *};

#[derive(Clone)]
struct Mdl {
    api: Api,
    search: search_bar::Mdl,
    results: Option<SearchResponse>,
    error: Option<String>,
    send_count: u64,
    recv_count: u64,
}

impl Default for Mdl {
    fn default() -> Self {
        let mut search = search_bar::Mdl::default();
        search.attrs = class!["search-bar"];
        search.clear_label = "x".to_string();
        Self {
            api: Api::new("/v0".to_string()),
            search,
            results: None,
            error: None,
            send_count: 0,
            recv_count: 0,
        }
    }
}

enum Msg {
    Search(search_bar::Msg),
    SearchResponse(fetch::Result<SearchResponse>, u64),
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Mdl {
    Mdl::default()
}

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Search(msg) => match msg {
            search_bar::Msg::Search(txt) => {
                mdl.search.search_term = txt.clone();
                let api = mdl.api.clone();
                let bbox = MapBbox {
                    sw: MapPoint {
                        lat: 40.0,
                        lng: -20.0,
                    },
                    ne: MapPoint {
                        lat: 60.0,
                        lng: 20.0,
                    },
                };
                mdl.send_count += 1;
                let c = mdl.send_count;
                orders.perform_cmd(
                    async move { Msg::SearchResponse(api.search(&txt, &bbox).await, c) },
                );
            }
            search_bar::Msg::Clear => {
                mdl.search.search_term = String::new();
                mdl.results = None;
            }
        },
        Msg::SearchResponse(res, c) => match res {
            Ok(x) => {
                seed::log!(
                    "found",
                    x.visible.len() + x.invisible.len(),
                    "places",
                    "last sent:",
                    mdl.send_count,
                    "last received:",
                    mdl.recv_count,
                    "current:",
                    c
                );
                if c > mdl.recv_count {
                    mdl.recv_count = c;
                    mdl.results = Some(x);
                } else {
                    seed::log("outdated result");
                }
            }
            Err(err) => {
                seed::error!(err);
                mdl.error = Some(format!("{:?}", err));
            }
        },
    }
}

fn view(mdl: &Mdl) -> Node<Msg> {
    let res_item_cfg = search_result_item::Mdl::default();

    div![
        h1!["OpenFairDB search"],
        search_bar::view(&mdl.search).map_msg(Msg::Search),
        if let Some(res) = &mdl.results {
            let vis = res
                .visible
                .iter()
                .map(|p| li![search_result_item::view(&res_item_cfg, p)]);
            let invis = res
                .invisible
                .iter()
                .map(|p| search_result_item::view(&res_item_cfg, p));
            div![
                class!["search-results"],
                p!["Suchergebnisse:", ul![vis], ul![invis]]
            ]
        } else {
            empty!()
        }
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
