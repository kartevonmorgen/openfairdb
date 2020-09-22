use super::{address_to_html, leaflet_css_link, map_scripts, page};
use crate::core::prelude::*;
use maud::{html, Markup};
use std::collections::HashMap;

type Ratings = Vec<(Rating, Vec<Comment>)>;

pub struct EntryPresenter {
    pub place: Place,
    pub ratings: HashMap<RatingContext, Ratings>,
    pub allow_archiving: bool,
}

impl From<(Place, Vec<(Rating, Vec<Comment>)>, Role)> for EntryPresenter {
    fn from((place, rtngs, role): (Place, Vec<(Rating, Vec<Comment>)>, Role)) -> EntryPresenter {
        let mut p: EntryPresenter = (place, rtngs).into();
        p.allow_archiving = matches!(role, Role::Admin | Role::Scout);
        p
    }
}

impl From<(Place, Vec<(Rating, Vec<Comment>)>)> for EntryPresenter {
    fn from((place, rtngs): (Place, Vec<(Rating, Vec<Comment>)>)) -> EntryPresenter {
        let mut ratings: HashMap<RatingContext, Ratings> = HashMap::new();

        for (r, comments) in rtngs {
            if let Some(x) = ratings.get_mut(&r.context) {
                x.push((r, comments));
            } else {
                ratings.insert(r.context, vec![(r, comments)]);
            }
        }
        let allow_archiving = false;
        EntryPresenter {
            place,
            ratings,
            allow_archiving,
        }
    }
}

pub fn entry(email: Option<&str>, e: EntryPresenter) -> Markup {
    page(
        &format!("{} | OpenFairDB", e.place.title),
        email,
        None,
        Some(leaflet_css_link()),
        entry_detail(e),
    )
}

fn entry_detail(e: EntryPresenter) -> Markup {
    let rev = format!("v{}", u64::from(e.place.revision));
    html! {
        h3 {
            (e.place.title)
            " "
            span class="rev" {
                "("
                @if e.allow_archiving {
                     a href=(format!("/places/{}/history", e.place.id)) { (rev) }
                     " | "
                     a href=(format!("/places/{}/review", e.place.id)) { "review" }
                } @else {
                    (rev)
                }
                ")"
            }
        }
        p {(e.place.description)}
        p {
            table {
                @if let Some(ref l) = e.place.links {
                    @if let Some(ref h) = l.homepage {
                        tr {
                            td { "Homepage" }
                            td { a href=(h) { (h) } }
                        }
                    }
                }
                @if let Some(ref c) = e.place.contact {
                    @if let Some(ref m) = c.email {
                        tr {
                            td { "eMail" }
                            td { a href=(format!("mailto:{}",m)) { (m) } }
                        }
                    }
                    @if let Some(ref t) = c.phone {
                        tr {
                            td { "Phone" }
                            td { a href=(format!("tel:{}",t)) { (t) } }
                        }
                    }
                }
                @if let Some(ref a) = e.place.location.address {
                    @if !a.is_empty() {
                        tr {
                            td { "Address" }
                            td { (address_to_html(&a)) }
                        }
                    }
                }
            }
        }
        p {
            ul {
                @for t in &e.place.tags{
                    li{ (format!("#{}", t)) }
                }
            }
        }
        h3 { "Ratings" }

        @for (ctx, ratings) in e.ratings {
            h4 { (format!("{:?}",ctx)) }
            ul {
                @for (r,comments) in ratings {
                    li {
                        (rating(e.place.id.as_ref(), e.allow_archiving, &r, &comments))
                    }
                }
            }
        }
        div id="map" style="height:300px;" { }
        (map_scripts(&[e.place.into()]))
    }
}

fn rating(place_id: &str, archive: bool, r: &Rating, comments: &[Comment]) -> Markup {
    html! {
      h5 { (r.title) " " span { (format!("({})",i8::from(r.value))) } }
      @if archive {
        form action = "/ratings/actions/archive" method = "POST" {
            input type="hidden" name="ids" value=(r.id.to_string());
            input type="hidden" name="place_id" value=(place_id);
            input type="submit" value="archive rating";
        }
      }
      @if let Some(ref src) = r.source {
          p { (format!("source: {}",src)) }
      }
      ul {
          @for c in comments {
              li {
                  p { (c.text) }
                  @if archive {
                    form action = "/comments/actions/archive" method = "POST" {
                        input type="hidden" name="ids" value=(c.id.to_string());
                        input type="hidden" name="place_id" value=(place_id);
                        input type="submit" value="archive comment";
                    }
                  }
              }
          }
      }
    }
}
