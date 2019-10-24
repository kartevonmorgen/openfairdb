use crate::core::{
    prelude::*,
    util::{
        filter::{self, InBBox},
        geo::MapBbox,
    },
};
use chrono::prelude::*;

pub fn query_events<D: Db>(
    db: &D,
    tags: Option<Vec<String>>,
    bbox: Option<MapBbox>,
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

    let mut events = db.get_events(start_min.map(Into::into), start_max.map(Into::into))?;

    if let Some(bbox) = bbox.as_ref().map(filter::extend_bbox) {
        events = events.into_iter().filter(|x| x.in_bbox(&bbox)).collect();
    }

    if let Some(tags) = tags {
        events = events
            .into_iter()
            .filter(|e| tags.iter().any(|t| e.tags.iter().any(|e_t| e_t == t)))
            .collect();
    }

    if let Some(ref email) = created_by {
        if let Some(user) = db.try_get_user_by_email(email)? {
            events = events
                .into_iter()
                .filter(|e| e.created_by.as_ref() == Some(&user.email))
                .collect();
        } else {
            events = vec![];
        }
    }

    events.sort_by(|a, b| a.start.cmp(&b.start));

    Ok(events)
}
