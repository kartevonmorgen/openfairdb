use crate::core::{prelude::*, util::filter::{self, InBBox}};
use chrono::prelude::*;

pub fn query_events<D: Db>(
    db: &D,
    tags: Option<Vec<String>>,
    bbox: Option<Bbox>,
    start_min: Option<NaiveDateTime>,
    start_max: Option<NaiveDateTime>,
    created_by: Option<String>,
    token: Option<String>,
) -> Result<Vec<Event>> {
    let _org = if let Some(ref token) = token {
        let org = db.get_org_by_api_token(token).map_err(|e| match e {
            RepoError::NotFound => Error::Parameter(ParameterError::Unauthorized),
            _ => Error::Repo(e),
        })?;
        Some(org)
    } else {
        None
    };

    let mut events = db.all_events()?;

    if let Some(bbox) = bbox {
        let ext_bbox = filter::extend_bbox(&bbox);
        events = events.into_iter().filter(|x| x.in_bbox(&ext_bbox)).collect();
    }

    if let Some(min) = start_min {
        events = events.into_iter().filter(|e| e.start >= min).collect();
    }
    if let Some(max) = start_max {
        events = events.into_iter().filter(|e| e.start <= max).collect();
    }
    if let Some(tags) = tags {
        events = events
            .into_iter()
            .filter(|e| tags.iter().any(|t| e.tags.iter().any(|e_t| e_t == t)))
            .collect();
    }

    if let Some(email) = created_by {
        let users = db.all_users()?;
        match users.into_iter().find(|u| u.email == *email) {
            Some(user) => {
                let u = Some(user.username);
                events = events.into_iter().filter(|e| e.created_by == u).collect();
            }
            None => {
                events = vec![];
            }
        }
    }
    events.sort_by(|a, b| a.start.cmp(&b.start));
    Ok(events)
}
