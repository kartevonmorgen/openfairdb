use super::prelude::*;

pub fn archive_events<R>(repo: &R, ids: &[&str]) -> Result<usize>
where
    R: EventRepo,
{
    log::debug!("Archiving events {:?}", ids);
    let archived = Timestamp::now();
    Ok(repo.archive_events(ids, archived)?)
}
