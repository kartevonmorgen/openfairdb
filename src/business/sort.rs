use entities::*;
use std::cmp::Ordering;
use super::geo;
use std::collections::HashMap;

trait DistanceTo {
    fn distance_to(&self, &Coordinate) -> f64;
}

impl DistanceTo for Entry {
    fn distance_to(&self, c: &Coordinate) -> f64 {
        geo::distance(
            &Coordinate {
                lat: self.lat,
                lng: self.lng,
            },
            c,
        )
    }
}

pub trait SortByDistanceTo {
    fn sort_by_distance_to(&mut self, &Coordinate);
}

impl SortByDistanceTo for Vec<Entry> {
    fn sort_by_distance_to(&mut self, c: &Coordinate) {
        if !(c.lat.is_finite() && c.lng.is_finite()) {
            return;
        }
        self.sort_by(|a, _| {
            if a.lat.is_finite() && a.lng.is_finite() {
                Ordering::Less
            } else {
                warn!("invalid coordinate: {}/{}", a.lat, a.lng);
                Ordering::Greater
            }
        });
        self.sort_by(|a, b| {
            a.distance_to(c)
                .partial_cmp(&b.distance_to(c))
                .unwrap_or(Ordering::Equal)
        })
    }
}

pub trait Rated {
    fn avg_rating(&self, &[Rating]) -> f64;
}

impl Rated for Entry {
    fn avg_rating(&self, ratings: &[Rating]) -> f64 {
        use self::RatingContext::*;

        let ratings_for_entry: Vec<&Rating> =
            ratings.iter().filter(|r| r.entry_id == self.id).collect();

        let avg_ratings = vec![
            avg_rating_for_context(&ratings_for_entry, &Diversity),
            avg_rating_for_context(&ratings_for_entry, &Renewable),
            avg_rating_for_context(&ratings_for_entry, &Fairness),
            avg_rating_for_context(&ratings_for_entry, &Humanity),
            avg_rating_for_context(&ratings_for_entry, &Transparency),
            avg_rating_for_context(&ratings_for_entry, &Solidarity),
        ];

        let sum = avg_ratings
            .iter()
            .fold(0.0, |acc, &r| acc + r.unwrap_or(0.0));
        let num_rated_contexts = avg_ratings
            .iter()
            .fold(0, |acc, &r| acc + if r.is_some() { 1 } else { 0 });

        if num_rated_contexts > 0 {
            sum / 6.0
        } else {
            0.0
        }
    }
}

fn avg_rating_for_context(ratings: &[&Rating], context: &RatingContext) -> Option<f64> {
    let applicable_ratings: Vec<&&Rating> = ratings
        .iter()
        .filter(|rating| rating.context == *context)
        .collect();

    let sum = applicable_ratings
        .iter()
        .fold(0_i64, |acc, rating| acc + i64::from(rating.value)) as f64;
    let n = applicable_ratings.len();

    let avg = sum / n as f64;
    if avg.is_nan() {
        None
    } else {
        Some(avg as f64)
    }
}

pub trait SortByAverageRating {
    fn calc_avg_ratings(&self, &[Rating]) -> HashMap<String, f64>;
    fn sort_by_avg_rating(&mut self, avg_ratings: &HashMap<String, f64>);
}

impl SortByAverageRating for Vec<Entry> {
    fn calc_avg_ratings(&self, ratings: &[Rating]) -> HashMap<String, f64> {
        self.iter()
            .map(|e| (e.id.clone(), e.avg_rating(ratings)))
            .collect()
    }

    fn sort_by_avg_rating(&mut self, avg_ratings: &HashMap<String, f64>) {
        self.sort_by(|a, b| {
            avg_ratings
                .get(&b.id)
                .unwrap_or_else(|| &0.0)
                .partial_cmp(avg_ratings.get(&a.id).unwrap_or_else(|| &0.0))
                .unwrap_or(Ordering::Equal)
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use test::Bencher;
    use uuid::Uuid;
    use business::builder::EntryBuilder;

    fn new_entry(id: &str, lat: f64, lng: f64) -> Entry {
        Entry::build().id(id).lat(lat).lng(lng).finish()
    }

    fn new_rating(id: &str, entry_id: &str, value: i8, context: RatingContext) -> Rating {
        Rating {
            id: id.into(),
            entry_id: entry_id.into(),
            created: 0,
            title: "blubb".into(),
            value: value.into(),
            context: context,
            source: Some("blabla".into()),
        }
    }

    #[test]
    fn test_average_rating() {
        let entry1 = new_entry("a", 0.0, 0.0);
        let entry2 = new_entry("b", 0.0, 0.0);
        let entry3 = new_entry("c", 0.0, 0.0);

        let ratings = vec![
            new_rating("1", "a", 0, RatingContext::Diversity),
            new_rating("2", "a", 0, RatingContext::Diversity),
            new_rating("3", "a", 3, RatingContext::Diversity),
            new_rating("4", "a", 3, RatingContext::Diversity),
            new_rating("5", "b", -3, RatingContext::Diversity),
            new_rating("6", "b", 3, RatingContext::Diversity),
        ];

        assert_eq!(entry1.avg_rating(&ratings), 0.25);
        assert_eq!(entry2.avg_rating(&ratings), 0.0);
        assert_eq!(entry3.avg_rating(&ratings), 0.0);
    }

    #[test]
    fn test_average_rating_different_contexts() {
        let entry1 = new_entry("a", 0.0, 0.0);
        let entry2 = new_entry("b", 0.0, 0.0);

        let ratings = vec![
            new_rating("1", "a", 0, RatingContext::Diversity),
            new_rating("2", "a", 10, RatingContext::Renewable),
            new_rating("3", "a", 7, RatingContext::Fairness),
            new_rating("4", "a", 9, RatingContext::Fairness),
            new_rating("5", "b", -3, RatingContext::Diversity),
            new_rating("6", "b", 3, RatingContext::Fairness),
        ];

        assert_eq!(entry1.avg_rating(&ratings), 3.0);
        assert_eq!(entry2.avg_rating(&ratings), 0.0);
    }

    #[test]
    fn test_sort_by_avg_rating() {
        let mut entries = vec![
            new_entry("a", 0.0, 0.0),
            new_entry("b", 0.0, 0.0),
            new_entry("c", 0.0, 0.0),
            new_entry("d", 0.0, 0.0),
            new_entry("e", 0.0, 0.0),
        ];

        let ratings = vec![
            new_rating("1", "b", 0, RatingContext::Diversity),
            new_rating("2", "b", 10, RatingContext::Diversity),
            new_rating("3", "c", 3, RatingContext::Diversity),
            new_rating("4", "d", -1, RatingContext::Diversity),
            new_rating("5", "e", 0, RatingContext::Diversity),
        ];

        let avg_ratings = entries.calc_avg_ratings(&ratings);
        entries.sort_by_avg_rating(&avg_ratings);

        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "c");
        assert!(entries[2].id == "a" || entries[2].id == "e");
        assert!(entries[3].id == "a" || entries[3].id == "e");
        assert_eq!(entries[4].id, "d");

        // tests:
        // - negative ratings
    }

    #[test]
    fn test_sort_by_avg_rating_with_no_ratings() {
        let mut entries = vec![
            new_entry("a", 0.0, 0.0),
            new_entry("b", 0.0, 0.0),
            new_entry("c", 0.0, 0.0),
            new_entry("d", 0.0, 0.0),
            new_entry("e", 0.0, 0.0),
        ];
        let ratings = vec![];
        let avg_ratings = entries.calc_avg_ratings(&ratings);
        entries.sort_by_avg_rating(&avg_ratings);

        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[1].id, "b");
        assert_eq!(entries[2].id, "c");
        assert_eq!(entries[3].id, "d");
        assert_eq!(entries[4].id, "e");
    }

    #[test]
    fn sort_by_distance() {
        let mut entries = vec![
            new_entry("a", 1.0, 0.0),
            new_entry("b", 0.0, 0.0),
            new_entry("c", 1.0, 1.0),
            new_entry("d", 0.0, 0.5),
            new_entry("e", -1.0, -1.0),
        ];
        let x = Coordinate { lat: 0.0, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "d");
        assert_eq!(entries[2].id, "a");
        assert!(entries[3].id == "c" || entries[3].id == "e");
        assert!(entries[4].id == "c" || entries[4].id == "e");
    }

    use std::f64::{INFINITY, NAN};

    #[test]
    fn sort_with_invalid_coordinates() {
        let mut entries = vec![
            new_entry("a", 1.0, NAN),
            new_entry("b", 1.0, INFINITY),
            new_entry("c", 2.0, 0.0),
            new_entry("d", NAN, NAN),
            new_entry("e", 1.0, 0.0),
        ];
        let x = Coordinate { lat: 0.0, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "e");
        assert_eq!(entries[1].id, "c");

        let mut entries = vec![
            new_entry("a", 2.0, 0.0),
            new_entry("b", 0.0, 0.0),
            new_entry("c", 1.0, 0.0),
        ];

        let x = Coordinate { lat: NAN, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[1].id, "b");
        assert_eq!(entries[2].id, "c");
    }

    pub fn create_entries_with_ratings(n: usize) -> (Vec<Entry>, Vec<Rating>) {
        let entries: Vec<Entry> = (0..n).map(|_| Entry::build().finish()).collect();

        let ratings: Vec<_> = entries
            .iter()
            .map(|e| {
                let ratings = create_ratings_for_entry(&e.id, 1);
                ratings[0].clone()
            })
            .collect();

        (entries, ratings)
    }

    fn create_entry_with_multiple_ratings(n: usize) -> (Entry, Vec<Rating>) {
        let entry = Entry::build().finish();
        let ratings = create_ratings_for_entry(&entry.id, n);
        (entry, ratings)
    }

    fn create_ratings_for_entry(id: &str, n: usize) -> Vec<Rating> {
        (0..n)
            .map(|_| Rating {
                id: Uuid::new_v4().simple().to_string(),
                entry_id: id.into(),
                created: 0,
                title: "".into(),
                value: 2,
                context: RatingContext::Diversity,
                source: None,
            })
            .collect()
    }

    #[bench]
    fn bench_for_sorting_1000_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings) = create_entries_with_ratings(1000);
        let avg_ratings = entries.calc_avg_ratings(&ratings);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&avg_ratings);
        });
    }

    #[ignore]
    #[bench]
    fn bench_for_sorting_10_000_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings) = create_entries_with_ratings(10_000);
        let avg_ratings = entries.calc_avg_ratings(&ratings);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&avg_ratings);
        });
    }

    #[ignore]
    #[bench]
    fn bench_for_sorting_100_000_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings) = create_entries_with_ratings(100_000);
        let avg_ratings = entries.calc_avg_ratings(&ratings);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&avg_ratings);
        });
    }

    #[bench]
    fn bench_calc_avg_of_1000_ratings_for_an_entry(b: &mut Bencher) {
        let (entry, ratings) = create_entry_with_multiple_ratings(1000);
        b.iter(|| entry.avg_rating(&ratings));
    }

    #[bench]
    fn bench_calc_avg_of_100_ratings_for_a_rating_context(b: &mut Bencher) {
        let (_, ratings) = create_entry_with_multiple_ratings(100);
        let ratings: Vec<_> = ratings.iter().collect();
        b.iter(|| avg_rating_for_context(&ratings, &RatingContext::Diversity));
    }

    #[bench]
    fn bench_calc_avg_of_1000_ratings_for_a_rating_context(b: &mut Bencher) {
        let (_, ratings) = create_entry_with_multiple_ratings(1000);
        let ratings: Vec<_> = ratings.iter().collect();
        b.iter(|| avg_rating_for_context(&ratings, &RatingContext::Diversity));
    }
}
