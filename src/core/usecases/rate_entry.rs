use chrono::*;
use core::prelude::*;
use uuid::Uuid;

#[cfg_attr(rustfmt, rustfmt_skip)]
#[derive(Deserialize, Debug, Clone)]
pub struct RateEntry {
    pub entry   : String,
    pub title   : String,
    pub value   : i8,
    pub context : RatingContext,
    pub comment : String,
    pub source  : Option<String>,
    pub user    : Option<String>,
}

pub fn rate_entry<D: Db>(db: &mut D, r: RateEntry) -> Result<()> {
    let e = db.get_entry(&r.entry)?;
    if r.comment.len() < 1 {
        return Err(Error::Parameter(ParameterError::EmptyComment));
    }
    if r.value > 2 || r.value < -1 {
        return Err(Error::Parameter(ParameterError::RatingValue));
    }
    let now = Utc::now().timestamp() as u64;
    let rating_id = Uuid::new_v4().to_simple_ref().to_string();
    let comment_id = Uuid::new_v4().to_simple_ref().to_string();
    #[cfg_attr(rustfmt, rustfmt_skip)]
    db.create_rating(&Rating{
        id       : rating_id.clone(),
        entry_id : e.id,
        created  : now,
        title    : r.title,
        value    : r.value,
        context  : r.context,
        source   : r.source
    })?;
    #[cfg_attr(rustfmt, rustfmt_skip)]
    db.create_comment(&Comment {
        id: comment_id.clone(),
        created: now,
        text: r.comment,
        rating_id,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::super::*;
    use super::*;

    #[test]
    fn rate_non_existing_entry() {
        let mut db = MockDb::new();
        assert!(
            rate_entry(
                &mut db,
                RateEntry {
                    entry: "does_not_exist".into(),
                    title: "title".into(),
                    comment: "a comment".into(),
                    context: RatingContext::Fairness,
                    user: None,
                    value: 2,
                    source: Some("source".into()),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn rate_with_empty_comment() {
        let mut db = MockDb::new();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e];
        assert!(
            rate_entry(
                &mut db,
                RateEntry {
                    entry: "foo".into(),
                    comment: "".into(),
                    title: "title".into(),
                    context: RatingContext::Fairness,
                    user: None,
                    value: 2,
                    source: Some("source".into()),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn rate_with_invalid_value_comment() {
        let mut db = MockDb::new();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e];
        assert!(
            rate_entry(
                &mut db,
                RateEntry {
                    entry: "foo".into(),
                    comment: "comment".into(),
                    title: "title".into(),
                    context: RatingContext::Fairness,
                    user: None,
                    value: 3,
                    source: Some("source".into()),
                },
            )
            .is_err()
        );
        assert!(
            rate_entry(
                &mut db,
                RateEntry {
                    entry: "foo".into(),
                    title: "title".into(),
                    comment: "comment".into(),
                    context: RatingContext::Fairness,
                    user: None,
                    value: -2,
                    source: Some("source".into()),
                },
            )
            .is_err()
        );
    }

    #[test]
    fn rate_without_login() {
        let mut db = MockDb::new();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e];
        assert!(
            rate_entry(
                &mut db,
                RateEntry {
                    entry: "foo".into(),
                    comment: "comment".into(),
                    title: "title".into(),
                    context: RatingContext::Fairness,
                    user: None,
                    value: 2,
                    source: Some("source".into()),
                },
            )
            .is_ok()
        );

        assert_eq!(db.ratings.len(), 1);
        assert_eq!(db.comments.len(), 1);
        assert_eq!(db.ratings[0].entry_id, "foo");
        assert_eq!(db.comments[0].rating_id, db.ratings[0].id);
    }

}
