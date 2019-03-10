use crate::core::prelude::*;

pub trait Rated {
    fn avg_ratings(&self, _: &[Rating]) -> AvgRatings;
}

impl Rated for Entry {
    fn avg_ratings(&self, ratings: &[Rating]) -> AvgRatings {
        debug_assert_eq!(
            ratings.len(),
            ratings.iter().filter(|r| r.entry_id == self.id).count()
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
    use crate::test::Bencher;
    use uuid::Uuid;

    fn new_entry(id: &str) -> Entry {
        Entry::build().id(id).finish()
    }

    fn new_rating(id: &str, entry_id: &str, value: i8, context: RatingContext) -> Rating {
        Rating {
            id: id.into(),
            entry_id: entry_id.into(),
            created: Timestamp::now(),
            archived: None,
            title: "blubb".into(),
            value: value.into(),
            context,
            source: Some("blabla".into()),
        }
    }

    #[test]
    fn test_average_rating() {
        let entry1 = new_entry("a");
        let entry2 = new_entry("b");
        let entry3 = new_entry("c");

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
        let entry1 = new_entry("a");
        let entry2 = new_entry("b");

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

    pub fn create_entries_with_ratings(n: usize) -> (Vec<Entry>, Vec<Rating>) {
        let entries: Vec<Entry> = (0..n).map(|_| Entry::build().finish()).collect();

        let ratings: Vec<_> = entries
            .iter()
            .map(|e| {
                let ratings = create_ratings_of_entry(&e.id, 1);
                ratings[0].clone()
            })
            .collect();

        (entries, ratings)
    }

    fn create_entry_with_multiple_ratings(n: usize) -> (Entry, Vec<Rating>) {
        let entry = Entry::build().finish();
        let ratings = create_ratings_of_entry(&entry.id, n);
        (entry, ratings)
    }

    fn create_ratings_of_entry(id: &str, n: usize) -> Vec<Rating> {
        (0..n)
            .map(|_| Rating {
                id: Uuid::new_v4().to_simple_ref().to_string(),
                entry_id: id.into(),
                created: Timestamp::now(),
                archived: None,
                title: "".into(),
                value: 2.into(),
                context: RatingContext::Diversity,
                source: None,
            })
            .collect()
    }

    #[bench]
    fn bench_calc_avg_of_1000_ratings_for_an_entry(b: &mut Bencher) {
        let (entry, ratings) = create_entry_with_multiple_ratings(1000);
        b.iter(|| entry.avg_ratings(&ratings[..]));
    }
}
