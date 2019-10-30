use crate::core::prelude::*;
use uuid::Uuid;

#[rustfmt::skip]
#[derive(Deserialize, Debug, Clone)]
pub struct RateEntry {
    pub entry   : String,
    pub title   : String,
    pub value   : RatingValue,
    pub context : RatingContext,
    pub comment : String,
    pub source  : Option<String>,
    pub user    : Option<String>,
}

#[derive(Debug, Clone)]
pub struct Storable(Entry, Rating, Comment);

impl Storable {
    pub fn rating_id(&self) -> &str {
        &self.1.id
    }
    pub fn comment_id(&self) -> &str {
        &self.2.id
    }
}

pub fn prepare_new_rating<D: Db>(db: &D, r: RateEntry) -> Result<Storable> {
    if r.comment.is_empty() {
        return Err(Error::Parameter(ParameterError::EmptyComment));
    }
    if !r.value.is_valid() {
        return Err(Error::Parameter(ParameterError::RatingValue));
    }
    let now = Timestamp::now();
    let rating_id = Uuid::new_v4().to_simple_ref().to_string();
    let comment_id = Uuid::new_v4().to_simple_ref().to_string();
    let entry = db.get_entry(&r.entry)?;
    debug_assert_eq!(entry.uid.as_ref(), &r.entry);
    let rating = Rating {
        id: rating_id.clone(),
        entry_id: r.entry,
        created: now,
        archived: None,
        title: r.title,
        value: r.value,
        context: r.context,
        source: r.source,
    };
    let comment = Comment {
        id: comment_id,
        created: now,
        archived: None,
        text: r.comment,
        rating_id: rating_id.clone(),
    };
    Ok(Storable(entry, rating, comment))
}

pub fn store_new_rating<D: Db>(db: &D, s: Storable) -> Result<(Entry, Vec<Rating>)> {
    let Storable(entry, rating, comment) = s;
    debug_assert_eq!(entry.uid.as_ref(), &rating.entry_id);
    debug_assert_eq!(rating.id, comment.rating_id);
    db.create_rating(rating)?;
    db.create_comment(comment)?;
    let ratings = db.load_ratings_of_entry(entry.uid.as_ref())?;
    Ok((entry, ratings))
}

#[cfg(test)]
mod tests {

    use super::super::tests::MockDb;
    use super::super::*;
    use super::*;

    #[test]
    fn rate_non_existing_entry() {
        let db = MockDb::default();
        assert!(prepare_new_rating(
            &db,
            RateEntry {
                entry: "does_not_exist".into(),
                title: "title".into(),
                comment: "a comment".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_with_empty_comment() {
        let mut db = MockDb::default();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e].into();
        assert!(prepare_new_rating(
            &db,
            RateEntry {
                entry: "foo".into(),
                comment: "".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_with_invalid_value_comment() {
        let mut db = MockDb::default();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e].into();
        assert!(prepare_new_rating(
            &db,
            RateEntry {
                entry: "foo".into(),
                comment: "comment".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(3),
                source: Some("source".into()),
            },
        )
        .is_err());
        assert!(prepare_new_rating(
            &db,
            RateEntry {
                entry: "foo".into(),
                title: "title".into(),
                comment: "comment".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(-2),
                source: Some("source".into()),
            },
        )
        .is_err());
    }

    #[test]
    fn rate_without_login() {
        let mut db = MockDb::default();
        let e = Entry::build().id("foo").finish();
        db.entries = vec![e].into();
        let c = prepare_new_rating(
            &db,
            RateEntry {
                entry: "foo".into(),
                comment: "comment".into(),
                title: "title".into(),
                context: RatingContext::Fairness,
                user: None,
                value: RatingValue::from(2),
                source: Some("source".into()),
            },
        )
        .unwrap();
        assert!(store_new_rating(&db, c).is_ok());

        assert_eq!(db.ratings.borrow().len(), 1);
        assert_eq!(db.comments.borrow().len(), 1);
        assert_eq!(db.ratings.borrow()[0].entry_id, "foo");
        assert_eq!(db.comments.borrow()[0].rating_id, db.ratings.borrow()[0].id);
    }
}
