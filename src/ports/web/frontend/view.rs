use crate::core::prelude::*;
use maud::{html, Markup};

const LEAFLET_CSS_URL: &str = "https://unpkg.com/leaflet@1.4.0/dist/leaflet.css";
const LEAFLET_CSS_SHA512: &str="sha512-puBpdR0798OZvTTbP4A8Ix/l+A4dHDD0DGqYW6RQ+9jxkRFclaxxQb/SJAWZfWAkuyeQUytO7+7N4QKrDh+drA==";
const LEAFLET_JS_URL: &str = "https://unpkg.com/leaflet@1.4.0/dist/leaflet.js";
const LEAFLET_JS_SHA512 : &str="sha512-QVftwZFqvtRNi0ZyCtsznlKSWOStnDORoefr1enyq5mVL4tmKB3S/EnC3rRJcxCPavG10IcrVGSmPh6Qw5lwrg==";

pub fn event(ev: Event) -> Markup {
    page(
        &ev.title,
        Some(html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="";
        }),
        html! {
            h2{ (ev.title) }
            h4 { "Zeit" }
            p{
                (ev.start.to_string())
                    @if let Some(end) = ev.end{
                        " bis "
                        (end.to_string())
                    }
            }
            h4 { "Beschreibung" }
            p{ (ev.description.unwrap_or("".to_string())) }

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
                    h4 {"Koordinaten"}
                    p{
                        (format!("{:.2} / {:.2}",location.lat, location.lng))
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
                    @if let Some(phone) = contact.telephone{
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
                h4{ "Anmeldung"}
                p {
                    @match reg{
                        RegistrationType::Email => "eMail" ,
                        RegistrationType::Phone => "Telefon",
                            RegistrationType::Homepage => "Webseite",
                    }
                }
            }
            p {
                h4{"Tags"}
                ul{
                    @for t in ev.tags{
                        li {(t)}
                    }
                }
            }

            @if let Some(location) = ev.location {
                div id="map" style="height:300px;" { }
                script
                    src=(LEAFLET_JS_URL)
                    integrity=(LEAFLET_JS_SHA512)
                    crossorigin=""{}
                script{
                    (format!("window.OFDB_EVENT_POS={{lat:{lat},lng:{lng}}};", lat=location.lat,lng=location.lng))
                }
                script src= "/frontend/event.js"{}
            }
        },
    )
}

pub fn events(events: &[Event]) -> Markup {
    page(
        "List of events",
        None,
        html! {
            h3{ "Events" }
            ul {
                @for e in events{
                    li{
                        a href=(format!("/frontend/events/{}",e.id)) {
                        (format!("{} - {}", e.start.to_string(), e.title))
                        }
                    }
                }
            }
        },
    )
}

fn page(title: &str, h: Option<Markup>, content: Markup) -> Markup {
    html! {
        head{
            title {(title)}
            @if let Some(h) = h {
               (h)
            }
        }
        body{
            (content)
        }
    }
}
