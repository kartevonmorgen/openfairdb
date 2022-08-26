use ofdb_seed::{boundary::TagFrequency, Api};
use seed::{prelude::*, *};
use std::cmp::Ordering;

const FETCH_LIMIT: usize = 1_000;
const FETCH_MAX: usize = 10_000;
const FETCH_STEP: usize = 1_000;
const SHOW_MAX: usize = 5;

struct Mdl {
    tags: Vec<TagFrequency>,
    input: String,
    filtered: Vec<TagFrequency>,
    api: Api,
    offset: usize,
}

impl Default for Mdl {
    fn default() -> Self {
        let api = Api::new("/v0".to_string());
        Self {
            tags: vec![],
            input: String::new(),
            filtered: vec![],
            offset: 0,
            api,
        }
    }
}

enum Msg {
    InputChange(String),
    TagSearchResponse(fetch::Result<Vec<TagFrequency>>),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Mdl {
    let mut mdl = Mdl::default();
    fetch_tags(&mut mdl, orders);
    mdl
}

fn fetch_tags(mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    if mdl.offset < FETCH_MAX {
        let api = mdl.api.clone();
        let offset = mdl.offset;
        orders.perform_cmd(async move {
            Msg::TagSearchResponse(
                api.get_most_popular_tags(None, None, Some(FETCH_LIMIT), Some(offset))
                    .await,
            )
        });
        mdl.offset += FETCH_STEP;
    }
}

fn update(msg: Msg, mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::InputChange(input) => {
            mdl.input = input;
            if mdl.input.is_empty() {
                mdl.filtered.clear();
            } else {
                update_list(mdl, orders);
            }
        }
        Msg::TagSearchResponse(Ok(tags)) => {
            mdl.tags.extend_from_slice(&tags);
            update_list(mdl, orders);
        }
        Msg::TagSearchResponse(Err(err)) => {
            error!("Could not fetch tags: {:?}", err);
        }
    }
}

fn update_list(mdl: &mut Mdl, orders: &mut impl Orders<Msg>) {
    if !mdl.input.is_empty() {
        mdl.filtered = filter(&mdl.input, &mdl.tags);
        if mdl.filtered.len() < SHOW_MAX {
            fetch_tags(mdl, orders);
        }
    }
}

fn filter(s: &str, tags: &[TagFrequency]) -> Vec<TagFrequency> {
    let s = s.to_lowercase();
    // 1. Take the 5 most popular tags
    let mut tags: Vec<_> = tags
        .iter()
        .filter(|t| t.0.contains(&s))
        .take(SHOW_MAX)
        .cloned()
        .collect();

    // 2. Sort
    tags.sort_by(|a, b| {
        match (a.0.starts_with(&s), b.0.starts_with(&s)) {
            // Show tags that starts with the user input first
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => b.1.cmp(&a.1), // Sort by popularity
        }
    });
    tags
}

fn view(mdl: &Mdl) -> Node<Msg> {
    let tags = mdl.filtered.iter().map(|t| {
        li![
            span![C!["hash"], "#"],
            span![C!["name"], &t.0,],
            " ",
            span![C!["count"], "(", &t.1, " entries)",]
        ]
    });
    div![
        h1!["OpenFairDB tag completion example"],
        input![
            input_ev(Ev::Input, Msg::InputChange),
            attrs! { At::Value => mdl.input; }
        ],
        if mdl.filtered.is_empty() {
            empty!()
        } else {
            ul![tags]
        }
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}
