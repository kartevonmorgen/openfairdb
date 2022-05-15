use maud::{html, Markup};

use super::*;

pub fn event(user: Option<User>, ev: Event) -> Markup {
    page(
        &ev.title,
        user.as_ref().map(|u| &*u.email),
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
                @if let Some(contact) = ev.contact{
                    @if !contact.is_empty(){
                        @if let Some(ref org) = &contact.name {
                            h4{"Veranstalter"}
                            p{(org)}
                        }
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
                @if let Some(user) = &user {
                    @match user.role {
                        Role::Admin | Role::Scout => {
                            form action=(format!("/events/{}/archive", ev.id)) method="POST" {
                                input type="submit" value="archive event";
                            }
                        }
                        _=> {}
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

pub fn events(email: Option<&str>, events: &[Event]) -> Markup {
    let locations: Vec<_> = events
        .iter()
        .filter_map(|e| e.location.as_ref())
        .map(|l| l.into())
        .collect();

    page(
        "List of Events",
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
                                a href=(format!("/events/{}", e.id)) {
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
                                            @if let Some(o) = e.organizer() {
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
