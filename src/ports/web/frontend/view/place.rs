use maud::{html, Markup};

use super::page;
use crate::core::prelude::*;

pub fn place_history(user: &User, h: &PlaceHistory) -> Markup {
    page(
        "Place History",
        Some(&user.email),
        None,
        None,
        html! {
            div class="revisions" {
                table {
                    thead {
                        tr {
                            th{ "Revision" }
                            th{ "Log"  }

                            th{ "Title" }
                            th{ "Description" }

                            th{ "Position" }

                            th{ "Street" }
                            th{ "ZIP" }
                            th{ "City" }
                            th{ "Country" }

                            th{ "E-Mail" }
                            th{ "Phone" }

                            th{ "Homepage" }
                            th{ "Image" }
                            th{ "Image Link" }

                            th{ "Tags" }
                        }
                    }
                    tbody {
                        @for (r,logs) in &h.revisions {
                            tr {
                                td{ (u64::from(r.revision)) }
                                td{
                                    ul class="log" {
                                        @for l in logs {
                                            li { (review_status_log(r.revision, l)) }
                                        }
                                    }
                                }

                                td{ (r.title) }
                                td{ (r.description) }

                                td{
                                    (format!("{:.2}/{:0.2}",
                                        r.location.pos.lat().to_deg(),
                                        r.location.pos.lng().to_deg()
                                    ))
                                }
                                @if let Some(a) = &r.location.address {
                                    td{ @if let Some(x) = &a.street  { (x) } }
                                    td{ @if let Some(x) = &a.zip     { (x) } }
                                    td{ @if let Some(x) = &a.city    { (x) } }
                                    td{ @if let Some(x) = &a.country { (x) } }
                                } @else {
                                    td{ }
                                    td{ }
                                    td{ }
                                    td{ }
                                }
                                @if let Some(c) = &r.contact {
                                    td{ @if let Some(x) = &c.email  { a href=(format!("mailto:{}",x)) { (x) } } }
                                    td{ @if let Some(x) = &c.phone  { (x) } }
                                } @else {
                                    td {}
                                    td {}
                                }
                                @if let Some(l) = &r.links {
                                    td{ @if let Some(x) = &l.homepage   { a href=(x) { (x) } } }
                                    td{ @if let Some(x) = &l.image      { img src=(x); } }
                                    td{ @if let Some(x) = &l.image_href { a href=(x) { (x) } } }
                                } @else {
                                    td {}
                                    td {}
                                    td {}
                                }
                                td{
                                    ul class="tags" {
                                        @for t in &r.tags {
                                            li { (t) }
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

fn review_status_log(place_rev: Revision, l: &ReviewStatusLog) -> Markup {
    use ReviewStatus as S;
    let status = match l.status {
        S::Rejected => "Rejected",
        S::Archived => "Archived",
        S::Created => {
            if place_rev.is_initial() {
                "Created"
            } else {
                "Modified"
            }
        }
        S::Confirmed => "Confirmed",
    };
    html! {
        span class="status" { (status) }
        " at "
        (l.activity.activity.at)
        " by "
        @if let Some(email) = &l.activity.activity.by {
            (email)
        } @else {
            "anonymous visitor"
        }
        @if let Some(c) = &l.activity.comment {
            span class="comment" { " \"" (c) "\"" }
        }
        @if let Some(c) = &l.activity.context {
            span class="context" { " (" (c) ")" }
        }
    }
}

pub fn place_review(email: &str, place: &Place, status: ReviewStatus) -> Markup {
    use ReviewStatus as S;
    let options = [
        (S::Rejected, "reject"),
        (S::Archived, "archive"),
        (S::Confirmed, "confirm"),
    ];
    page(
        "Place Review",
        Some(email),
        None,
        None,
        html! {
            div class="review" {
                h2 { (format!("Add Review for \"{}\"", place.title)) }
                form action=(format!("/places/{}/review", place.id)) method="POST" {
                    fieldset {
                        label {
                            "Comment:"
                            br;
                            input required? name="comment" placeholder="Comment";
                        }
                        br;
                        label { "Action:"
                            br;
                            select name="status" {
                                @for (o, label) in options.iter() {
                                    option value=(i16::from(*o)) disabled?[*o==status] { (label) }
                                }
                            }
                        }
                    }
                    input type="submit" value="change";
                }
            }
        },
    )
}
