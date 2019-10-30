use crate::core::prelude::*;
use maud::{html, Markup};
use num_traits::ToPrimitive;

const LEAFLET_CSS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.css";
const LEAFLET_CSS_SHA512: &str="sha512-puBpdR0798OZvTTbP4A8Ix/l+A4dHDD0DGqYW6RQ+9jxkRFclaxxQb/SJAWZfWAkuyeQUytO7+7N4QKrDh+drA==";
const LEAFLET_JS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.js";
const LEAFLET_JS_SHA512 : &str="sha512-QVftwZFqvtRNi0ZyCtsznlKSWOStnDORoefr1enyq5mVL4tmKB3S/EnC3rRJcxCPavG10IcrVGSmPh6Qw5lwrg==";
const MAP_JS_URL: &str = "/map.js";

mod dashboard;
mod entry;
mod login;
mod page;
mod password;
mod register;

pub use dashboard::*;
pub use entry::*;
pub use login::*;
use page::*;
pub use password::*;
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
                    placeholer="search term, empty = all";
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

pub fn search_results(email: Option<&str>, search_term: &str, entries: &[IndexedEntry]) -> Markup {
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

fn entry_result(e: &IndexedEntry) -> Markup {
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

pub fn event(email: Option<&str>, ev: Event) -> Markup {
    page(
        &ev.title,
        email,
        None,
        Some(html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="anonymous";
        }),
        html! {
            div class="details event" {
                div class="entity-type" { "Event"  }
                h2{ (ev.title) }
                p class="time" {
                    (ev.start.format("%d.%m.%Y %H:%M"))
                        @if let Some(end) = ev.end{
                            " - "
                            (end.format("%d.%m.%Y %H:%M"))
                        }
                }
                p class="description" { (ev.description.unwrap_or_default()) }

                @if let Some(ref location) = ev.location{
                        h4{ "Ort" }
                        @if let Some(ref addr) = location.address {
                            @if !addr.is_empty(){
                                @if let Some(ref s) = addr.street {
                                    (s) br;
                                }
                                @if let Some(ref z) = addr.zip {
                                    (z) " "
                                }
                                @if let Some(ref c) = addr.city {
                                    (c) br;
                                }
                                @if let Some(ref c) = addr.country {
                                    (c)
                                }
                            }
                        }
                        h4 { "Koordinaten" }
                        p{
                            (format!("{:.2} / {:.2}",location.pos.lat().to_deg(), location.pos.lng().to_deg()))
                        }
                }
                @if let Some(org) = ev.organizer {
                    h4{"Veranstalter"}
                    p{(org)}
                }
                @if let Some(contact) = ev.contact{
                    @if !contact.is_empty(){
                        h4{ "Kontakt" }
                        @if let Some(email) = contact.email{
                            (email)
                            br;
                        }
                        @if let Some(phone) = contact.phone{
                            (phone)
                        }
                    }
                }
                @if let Some(url) = ev.homepage{
                        h4{ "Webseite" }
                        p{
                            a href=(url) { (url) }
                        }
                }
                @if let Some(reg) = ev.registration{
                    h4{ "Anmeldung" }
                    p {
                        @match reg{
                            RegistrationType::Email => "eMail" ,
                            RegistrationType::Phone => "Telefon",
                            RegistrationType::Homepage => "Webseite",
                        }
                    }
                }
                p {
                    h4{ "Tags" }
                    ul class="tags" {
                        @for t in ev.tags{
                            li {(format!("#{}",t))}
                        }
                    }
                }

            }
            @if let Some(l) = ev.location {
                div id="map" style="height:100vh;" { }
                (map_scripts(&[l.into()]))
            }
        },
    )
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

impl From<Entry> for MapPin {
    fn from(e: Entry) -> Self {
        e.location.into()
    }
}

impl From<&Entry> for MapPin {
    fn from(e: &Entry) -> Self {
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

pub fn events(events: &[Event]) -> Markup {
    let locations: Vec<_> = events
        .iter()
        .filter_map(|e| e.location.as_ref())
        .map(|l| l.into())
        .collect();

    page(
        "List of Events",
        None,
        None,
        Some(html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="anonymous";
        }),
        html! {
            div class="events" {
                h3 { "Events" }
                @if events.is_empty() {
                    p class="no-results" {
                        "Es konnten keine Events gefunden werden."
                    }
                } @else {
                    ul class="event-list" {
                        @for e in events {
                            li {
                                a href=(format!("/events/{}", e.uid)) {
                                    div {
                                        h4 {
                                            span class="title" { (e.title) }
                                            " "
                                            span class="date" {
                                                (e.start.format("%d.%m.%y"))
                                            }
                                        }
                                        p {
                                            @if let Some(ref l) = e.location {
                                                @if let Some(ref a) = l.address {
                                                    @if let Some(ref city) = a.city {
                                                        span class="city" { (city) }
                                                        br;
                                                    }
                                                }
                                            }
                                            @if let Some(ref o) = e.organizer {
                                                span class="organizer" { (o) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div id="map" style="height:100vh;" { }
            (map_scripts(&locations))
        },
    )
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
