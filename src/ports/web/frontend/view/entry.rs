use super::{address_to_html, leaflet_css_link, map_scripts, page};
use crate::core::prelude::*;
use maud::{html, Markup};
use std::collections::HashMap;

type Ratings = Vec<(Rating, Vec<Comment>)>;

pub struct EntryPresenter {
    pub entry: Entry,
    pub ratings: HashMap<RatingContext, Ratings>,
    pub allow_archiving: bool,
}

impl From<(Entry, Vec<(Rating, Vec<Comment>)>, Role)> for EntryPresenter {
    fn from((entry, rtngs, role): (Entry, Vec<(Rating, Vec<Comment>)>, Role)) -> EntryPresenter {
        let mut p: EntryPresenter = (entry, rtngs).into();
        p.allow_archiving = match role {
            Role::Admin | Role::Scout => true,
            _ => false,
        };
        p
    }
}

impl From<(Entry, Vec<(Rating, Vec<Comment>)>)> for EntryPresenter {
    fn from((entry, rtngs): (Entry, Vec<(Rating, Vec<Comment>)>)) -> EntryPresenter {
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
            entry,
            ratings,
            allow_archiving,
        }
    }
}

pub fn entry(email: Option<&str>, e: EntryPresenter) -> Markup {
    page(
        &format!("{} | OpenFairDB", e.entry.title),
        email,
        None,
        Some(leaflet_css_link()),
        entry_detail(e),
    )
}

fn entry_detail(e: EntryPresenter) -> Markup {
    html! {
        h3 { (e.entry.title) }
        p {(e.entry.description)}
        p {
            table {
                @if let Some(ref h) = e.entry.homepage {
                    tr {
                        td { "Homepage" }
                        td { a href=(h) { (h) } }
                    }
                }
                @if let Some(ref c) = e.entry.contact {
                    @if let Some(ref m) = c.email {
                        tr {
                            td { "eMail" }
                            td { a href=(format!("mailto:{}",m)) { (m) } }
                        }
                    }
                    @if let Some(ref t) = c.telephone {
                        tr {
                            td { "Telephone" }
                            td { a href=(format!("tel:{}",t)) { (t) } }
                        }
                    }
                }
                @if let Some(ref a) = e.entry.location.address {
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
                @for t in &e.entry.tags{
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
                        (rating(&e.entry.id, e.allow_archiving, &r, &comments))
                    }
                }
            }
        }
        div id="map" style="height:300px;" { }
        (map_scripts(&[e.entry.into()]))
    }
}

fn rating(entry_id: &str, archive: bool, r: &Rating, comments: &[Comment]) -> Markup {
    html! {
      h5 { (r.title) " " span { (format!("({})",i8::from(r.value))) } }
      @if archive {
        form action = "/ratings/actions/archive" method = "POST" {
            input type="hidden" name="ids" value=(r.id);
            input type="hidden" name="entry_id" value=(entry_id);
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
                        input type="hidden" name="ids" value=(c.id);
                        input type="hidden" name="entry_id" value=(entry_id);
                        input type="submit" value="archive comment";
                    }
                  }
              }
          }
      }
    }
}
