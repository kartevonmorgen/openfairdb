//! Search bar

use seed::{prelude::*, *};

#[derive(Clone)]
pub struct Mdl {
    pub search_term: String,
    pub placeholder: Option<String>,
    pub attrs: Attrs,
    pub input_attrs: Attrs,
    pub clear_label: String,
}

impl Default for Mdl {
    fn default() -> Self {
        Self {
            search_term: String::new(),
            placeholder: Some("What are you looking for? (# for tags)".to_string()),
            attrs: attrs! {},
            input_attrs: attrs! {},
            clear_label: "clear".to_string(),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    Search(String),
    Clear,
}

pub fn view(mdl: &Mdl) -> Node<Msg> {
    div![
        &mdl.attrs,
        input![
            attrs! {
                At::Value => mdl.search_term;
                At::Type => "search";
            },
            mdl.placeholder
                .as_ref()
                .map(|p| attrs! { At::Placeholder => p; })
                .unwrap_or_else(|| attrs! {}),
            &mdl.input_attrs,
            input_ev(Ev::Input, Msg::Search),
            keyboard_ev(Ev::KeyUp, |ev| {
                ev.prevent_default();
                if ev.key().to_lowercase() == "escape" {
                    Some(Msg::Clear)
                } else {
                    None
                }
            })
        ],
        if !mdl.search_term.is_empty() {
            button![&mdl.clear_label, ev(Ev::Click, |_| Msg::Clear)]
        } else {
            empty!()
        }
    ]
}
