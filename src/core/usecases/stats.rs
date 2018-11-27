use crate::core::prelude::*;

#[derive(Debug)]
pub struct Stats {
    pub entry_count: u64,
    pub tag_count: u64,
    pub user_count: u64,
}

pub fn get_stats<D: Db>(db: &D) -> Result<Stats> {
    let entry_count = db.count_entries()?;
    let tag_count = db.count_tags()?;
    let user_count = db.count_users()?;
    Ok(Stats {
        entry_count,
        tag_count,
        user_count,
    })
}
