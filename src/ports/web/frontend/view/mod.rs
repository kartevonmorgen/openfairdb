use maud::{html, Markup};
use num_traits::ToPrimitive;

use crate::core::prelude::*;

const LEAFLET_CSS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.css";
const LEAFLET_CSS_SHA512: &str="sha512-puBpdR0798OZvTTbP4A8Ix/l+A4dHDD0DGqYW6RQ+9jxkRFclaxxQb/SJAWZfWAkuyeQUytO7+7N4QKrDh+drA==";
const LEAFLET_JS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.js";
const LEAFLET_JS_SHA512 : &str="sha512-QVftwZFqvtRNi0ZyCtsznlKSWOStnDORoefr1enyq5mVL4tmKB3S/EnC3rRJcxCPavG10IcrVGSmPh6Qw5lwrg==";
const MAP_JS_URL: &str = "/map.js";

mod dashboard;
mod entry;
mod event;
mod login;
mod page;
mod password;
mod place;
mod register;

pub use dashboard::*;
pub use entry::*;
pub use event::*;
pub use login::*;
use page::*;
pub use password::*;
pub use place::*;
pub use register::*;

pub fn index(email: Option<&str>) -> Markup {
    page(
        "OpenFairDB Search",
        email,
        None,
        None,
        html! {
            div class="search" {
                h1 {"OpenFairDB Search"}
                (global_search_form(None))
            }
        },
    )
}

pub fn global_search_form(search_term: Option<&str>) -> Markup {
    html! {
        div class="search-form" {
            form action="search" method="GET" {
                input
                    type="text"
                    name="q"
                    value=(search_term.unwrap_or(""))
                    size=(50)
                    maxlength=(200)
                    placeholder="search term, empty = all";
                br;
                input class="btn" type="submit" value="search";
            }
        }
    }
}

fn leaflet_css_link() -> Markup {
    html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="anonymous";
    }
}

pub fn search_results(email: Option<&str>, search_term: &str, entries: &[IndexedPlace]) -> Markup {
    page(
        "OpenFairDB Search Results",
        email,
        None,
        None,
        html! {
            div class="search" {
                h1 {"OpenFairDB Search"}
                (global_search_form(Some(search_term)))
            }
            div class="results" {
                @if entries.is_empty(){
                    p{ "We are so sorry but we could not find any entries
                        that are related to your search term "
                            em {
                                (format!("'{}'", search_term))
                            }
                    }
                } @else {
                    p {
                        ul class="result-list" {
                            @for e in entries {
                                li{
                                    (entry_result(e))
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

fn entry_result(e: &IndexedPlace) -> Markup {
    html! {
        h3 {
            a href=(format!("entries/{}",e.id)) {(e.title)}
        }
        p {(e.description)}
    }
}

fn address_to_html(addr: &Address) -> Markup {
    html! {
        @if let Some(ref s) = addr.street {
            (s) br;
        }
        @if let Some(ref z) = addr.zip {
            (z)
        }
        @if let Some(ref z) = addr.city {
            (z) br;
        }
        @if let Some(ref c) = addr.country {
            (c)
        }
    }
}

struct MapPin {
    lat: f64,
    lng: f64,
}

impl MapPin {
    fn to_js_object_string(&self) -> String {
        format!("{{lat:{},lng:{}}}", self.lat, self.lng)
    }
}

impl From<Place> for MapPin {
    fn from(e: Place) -> Self {
        e.location.into()
    }
}

impl From<&Place> for MapPin {
    fn from(e: &Place) -> Self {
        (&e.location).into()
    }
}

impl From<Location> for MapPin {
    fn from(l: Location) -> Self {
        (&l).into()
    }
}

impl From<&Location> for MapPin {
    fn from(l: &Location) -> Self {
        MapPin {
            lat: l.pos.lat().to_deg(),
            lng: l.pos.lng().to_deg(),
        }
    }
}

fn map_scripts(pins: &[MapPin]) -> Markup {
    let (center, zoom) = match pins.len() {
        1 => ((pins[0].lat, pins[0].lng), 13.0),
        _ => {
            //TODO: calculate center & zoom
            ((48.720, 9.152), 6.0)
        }
    };

    let center = format!("[{},{}]", center.0, center.1);

    let pins: String = pins
        .iter()
        .map(MapPin::to_js_object_string)
        .collect::<Vec<_>>()
        .join(",");

    html! {
      script{
        (format!("window.OFDB_MAP_PINS=[{}];window.OFDB_MAP_ZOOM={};OFDB_MAP_CENTER={};",
                 pins,
                 zoom,
                 center))
      }
      script
        src=(LEAFLET_JS_URL)
        integrity=(LEAFLET_JS_SHA512)
        crossorigin="anonymous" {}
      script src=(MAP_JS_URL){}
    }
}

pub fn user_search_result(admin_email: &str, users: &[User]) -> Markup {
    page(
        "Users",
        Some(admin_email),
        None,
        None,
        html! {
            main {
                h3 { "Users" }
                (search_users_form())
                @if users.is_empty() {
                    "No users were found :("
                } @else {
                    table {
                        thead {
                            tr {
                              th { "Username"        }
                              th { "eMail"           }
                              th { "eMail confirmed" }
                              th { "Role"            }
                              th { "Modify role"            }
                            }
                        }
                        tbody {
                            @for u in users {
                                tr {
                                    td { (u.email) }
                                    td { (if u.email_confirmed{"yes"}else{"no"}) }
                                    td { (format!("{:?}",u.role)) }
                                    td {
                                        @if u.email != admin_email {
                                            form action="change-user-role" method="POST" {
                                                select name = "role" required? {
                                                    option value="-1" {"-- please select --"}
                                                    option value=(Role::Guest.to_u8().unwrap()) { "Guest" }
                                                    option value=(Role::User.to_u8().unwrap())  { "User" }
                                                    option value=(Role::Scout.to_u8().unwrap()) { "Scout" }
                                                }
                                                input type="hidden" name="email" value=(u.email);
                                                input type="submit" value="change";
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn search_users_form() -> Markup {
    html! {
        form action="search-users" method="GET" {
            input type="email" name="email" placeholder="email address";
            br;
            input type="submit" value="search";
        }
    }
}
