use ofdb_entities::{place::*, rating::*};

pub trait Rated {
    fn avg_ratings(&self, _: &[Rating]) -> AvgRatings;
}

impl Rated for Place {
    fn avg_ratings(&self, ratings: &[Rating]) -> AvgRatings {
        debug_assert_eq!(
            ratings.len(),
            ratings.iter().filter(|r| r.place_id == self.id).count()
        );
        ratings
            .iter()
            .fold(AvgRatingsBuilder::default(), |mut acc, r| {
                acc.add(r.context, r.value);
                acc
            })
            .build()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use ofdb_entities::{builders::*, time::*};

    fn new_place(id: &str) -> Place {
        Place::build().id(id).finish()
    }

    fn new_rating(id: &str, place_id: &str, value: i8, context: RatingContext) -> Rating {
        Rating {
            id: id.into(),
            place_id: place_id.into(),
            created_at: Timestamp::now(),
            archived_at: None,
            title: "blubb".into(),
            value: value.into(),
            context,
            source: Some("blabla".into()),
        }
    }

    #[test]
    fn test_average_rating() {
        let entry1 = new_place("a");
        let entry2 = new_place("b");
        let entry3 = new_place("c");

        let ratings1 = [
            new_rating("1", "a", -1, RatingContext::Diversity),
            new_rating("2", "a", 1, RatingContext::Diversity),
            new_rating("3", "a", 2, RatingContext::Diversity),
            new_rating("4", "a", 1, RatingContext::Diversity),
        ];

        let ratings2 = [
            new_rating("5", "b", -1, RatingContext::Diversity),
            new_rating("6", "b", 1, RatingContext::Diversity),
        ];
        assert_eq!(entry1.avg_ratings(&ratings1).total(), 0.125.into());
        assert_eq!(entry2.avg_ratings(&ratings2).total(), 0.0.into());
        assert_eq!(entry3.avg_ratings(&[]).total(), 0.0.into());
    }

    #[test]
    fn test_average_rating_different_contexts() {
        let entry1 = new_place("a");
        let entry2 = new_place("b");

        let ratings1 = [
            new_rating("1", "a", -1, RatingContext::Diversity),
            new_rating("2", "a", 2, RatingContext::Renewable),
            new_rating("3", "a", 1, RatingContext::Fairness),
            new_rating("4", "a", 1, RatingContext::Renewable),
            new_rating("4", "a", 2, RatingContext::Fairness),
            new_rating("3", "a", 1, RatingContext::Diversity),
        ];

        let ratings2 = [
            new_rating("5", "b", -1, RatingContext::Diversity),
            new_rating("6", "b", 1, RatingContext::Fairness),
        ];

        assert_eq!(entry1.avg_ratings(&ratings1).total(), 0.5.into());
        assert_eq!(entry2.avg_ratings(&ratings2).total(), 0.0.into());
    }
}
