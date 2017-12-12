use entities::*;
use std::cmp::Ordering;
use super::geo;

trait DistanceTo {
    fn distance_to(&self, &Coordinate) -> f64;
}

impl DistanceTo for Entry {
    fn distance_to(&self, c: &Coordinate) -> f64 {
        geo::distance(&Coordinate { lat: self.lat, lng: self.lng, }, c)
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
        self.sort_by(|a, _|
            if a.lat.is_finite() && a.lng.is_finite() {
                Ordering::Less
            } else {
                warn!("invalid coordinate: {}/{}", a.lat, a.lng);
                Ordering::Greater
            }
        );
        self.sort_by(|a, b| a
            .distance_to(c)
            .partial_cmp(&b.distance_to(c))
            .unwrap_or(Ordering::Equal)
        )
    }
}

pub trait Rated {
    fn avg_rating(&self, &[Rating], &[Triple]) -> f64;
    fn avg_rating_for_context(&self, &[Rating], &[(&String, &String)], RatingContext) -> Option<f64>;
}

fn create_entry_ratings<'a>(id: &str, triples: &'a [Triple]) -> Vec<(&'a String, &'a String)> {
    triples
        .into_iter()
        .filter_map(|x| match *x {
            Triple {
                subject   : ObjectId::Entry(ref e_id),
                predicate : Relation::IsRatedWith,
                object    : ObjectId::Rating(ref r_id)
            } => Some((e_id, r_id)),
            _ => None
        })
        .filter(|entry_rating| *entry_rating.0 == id)
        .collect()
}

impl Rated for Entry {
    fn avg_rating(&self, ratings: &[Rating], triples: &[Triple]) -> f64 {
        let entry_ratings = create_entry_ratings(&self.id, triples);

        use self::RatingContext::*;

        let avg_ratings = vec![
            self.avg_rating_for_context(ratings, &entry_ratings, Diversity),
            self.avg_rating_for_context(ratings, &entry_ratings, Renewable),
            self.avg_rating_for_context(ratings, &entry_ratings, Fairness),
            self.avg_rating_for_context(ratings, &entry_ratings, Humanity),
            self.avg_rating_for_context(ratings, &entry_ratings, Transparency),
            self.avg_rating_for_context(ratings, &entry_ratings, Solidarity),
        ];

        let sum = avg_ratings.iter().fold(0.0, |acc, &r| acc + r.unwrap_or(0.0));
        let num_rated_contexts = avg_ratings.iter().fold(0, |acc, &r| acc + if r.is_some() {1} else {0});

        if num_rated_contexts > 0 {
            sum / 6.0
        } else {
            0.0
        }
    }

    fn avg_rating_for_context(&self, ratings: &[Rating], entry_ratings: &[(&String, &String)], context: RatingContext) -> Option<f64> {
        let applicable_ratings : Vec<&Rating> = ratings.into_iter()
            .filter_map(|rating| if rating.context == context
                && entry_ratings.iter()
                .any(|entry_rating| *entry_rating.1 == rating.id) { Some(rating) } else { None })
            .collect();

        let sum = applicable_ratings
            .iter()
            .fold(0, |acc, rating| acc + rating.value) as f64;
        let n = applicable_ratings.len();

        let avg = sum / n as f64;
        if avg.is_nan() {
            None
        } else {
            Some(avg as f64)
        }
    }
}

pub trait SortByAverageRating {
    fn sort_by_avg_rating(&mut self, &[Rating], &[Triple]);
}

impl SortByAverageRating for Vec<Entry> {
    fn sort_by_avg_rating(&mut self, ratings: &[Rating], triples: &[Triple]){
        self.sort_by(|a, b| b
            .avg_rating(ratings, triples)
            .partial_cmp(&a.avg_rating(ratings, triples))
            .unwrap_or(Ordering::Equal)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use uuid::Uuid;
    use business::builder::EntryBuilder;

    fn new_entry(id: &str, lat: f64, lng: f64) -> Entry {
        Entry::build()
            .id(id)
            .lat(lat)
            .lng(lng)
            .finish()
    }

    fn new_rating(id: &str, value: i8, context: RatingContext) -> Rating {
        Rating{
            id         : id.into(),
            created    : 0,
            title      : "blubb".into(),
            value      : value.into(),
            context    : context,
            source     : Some("blabla".into())
        }
    }

    #[test]
    fn test_average_rating() {
        let entry1 = new_entry("a", 0.0, 0.0);
        let entry2 = new_entry("b", 0.0, 0.0);
        let entry3 = new_entry("c", 0.0, 0.0);

        let ratings = vec![
            new_rating("1", 0, RatingContext::Diversity),
            new_rating("2", 0, RatingContext::Diversity),
            new_rating("3", 3, RatingContext::Diversity),
            new_rating("4", 3, RatingContext::Diversity),
            new_rating("5", -3, RatingContext::Diversity),
            new_rating("6", 3, RatingContext::Diversity),
        ];

        let triples = vec![
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("1".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("2".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("3".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("4".into())},
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("5".into())},
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("6".into())},
        ];

        assert_eq!(entry1.avg_rating(&ratings, &triples), 0.25);
        assert_eq!(entry2.avg_rating(&ratings, &triples), 0.0);
        assert_eq!(entry3.avg_rating(&ratings, &triples), 0.0);
    }

    #[test]
    fn test_average_rating_different_contexts() {
        let entry1 = new_entry("a", 0.0, 0.0);
        let entry2 = new_entry("b", 0.0, 0.0);

        let ratings = vec![
            new_rating("1", 0, RatingContext::Diversity),
            new_rating("2", 10, RatingContext::Renewable),
            new_rating("3", 7, RatingContext::Fairness),
            new_rating("4", 9, RatingContext::Fairness),
            new_rating("5", -3, RatingContext::Diversity),
            new_rating("6", 3, RatingContext::Fairness),
        ];

        let triples = vec![
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("1".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("2".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("3".into())},
            Triple{subject: ObjectId::Entry("a".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("4".into())},
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("5".into())},
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("6".into())},
        ];

        assert_eq!(entry1.avg_rating(&ratings, &triples), 3.0);
        assert_eq!(entry2.avg_rating(&ratings, &triples), 0.0);
    }

    #[test]
    fn test_sort_by_avg_rating(){
        let mut entries = vec![
            new_entry("a", 0.0, 0.0),
            new_entry("b", 0.0, 0.0),
            new_entry("c", 0.0, 0.0),
            new_entry("d", 0.0, 0.0),
            new_entry("e", 0.0, 0.0),
        ];

        let ratings = vec![
            new_rating("1", 0, RatingContext::Diversity),
            new_rating("2", 10, RatingContext::Diversity),
            new_rating("3", 3, RatingContext::Diversity),
            new_rating("4", -1, RatingContext::Diversity),
            new_rating("5", 0, RatingContext::Diversity),
        ];

        let triples = vec![
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("1".into())},
            Triple{subject: ObjectId::Entry("b".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("2".into())},
            Triple{subject: ObjectId::Entry("c".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("3".into())},
            Triple{subject: ObjectId::Entry("d".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("4".into())},
            Triple{subject: ObjectId::Entry("e".into()), predicate: Relation::IsRatedWith, object: ObjectId::Rating("5".into())},
        ];

        entries.sort_by_avg_rating(&ratings, &triples);


        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "c");
        assert!(entries[2].id == "a" || entries[2].id == "e");
        assert!(entries[3].id == "a" || entries[3].id == "e");
        assert_eq!(entries[4].id, "d");


        // tests:
        // - negative ratings
    }

    #[test]
    fn sort_by_distance() {
        let mut entries = vec![new_entry("a", 1.0, 0.0),
                               new_entry("b", 0.0, 0.0),
                               new_entry("c", 1.0, 1.0),
                               new_entry("d", 0.0, 0.5),
                               new_entry("e", -1.0, -1.0)];
        let x = Coordinate { lat: 0.0, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "d");
        assert_eq!(entries[2].id, "a");
        assert!(entries[3].id == "c" || entries[3].id == "e");
        assert!(entries[4].id == "c" || entries[4].id == "e");
    }

    use std::f64::{NAN, INFINITY};

    #[test]
    fn sort_with_invalid_coordinates() {
        let mut entries = vec![new_entry("a", 1.0, NAN),
                               new_entry("b", 1.0, INFINITY),
                               new_entry("c", 2.0, 0.0),
                               new_entry("d", NAN, NAN),
                               new_entry("e", 1.0, 0.0)];
        let x = Coordinate { lat: 0.0, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "e");
        assert_eq!(entries[1].id, "c");

        let mut entries = vec![new_entry("a", 2.0, 0.0),
                               new_entry("b", 0.0, 0.0),
                               new_entry("c", 1.0, 0.0)];

        let x = Coordinate { lat: NAN, lng: 0.0 };
        entries.sort_by_distance_to(&x);
        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[1].id, "b");
        assert_eq!(entries[2].id, "c");
    }


    fn create_entries_with_ratings_and_triples(n: usize) -> (Vec<Entry>, Vec<Rating>, Vec<Triple>) {

        let entries : Vec<Entry> = (0..n).map(|_| Entry::build().finish()).collect();

        let ratings_and_triples : Vec<_> = entries
            .iter()
            .map(|e|{
                let (ratings, triples) = create_ratings_for_entry(&e.id,1);
                (ratings[0].clone(),triples[0].clone())
            })
            .collect();

        let (ratings, triples) : (Vec<_>, Vec<_>) = ratings_and_triples.into_iter().unzip();

        (entries, ratings, triples)
    }

    fn create_entry_with_multiple_ratings_and_triples(n: usize) -> (Entry, Vec<Rating>, Vec<Triple>) {
        let entry = Entry::build().finish();
        let (ratings, triples) = create_ratings_for_entry(&entry.id,n);
        (entry, ratings, triples)
    }

    fn create_ratings_for_entry(id: &str, n: usize) -> (Vec<Rating>,Vec<Triple>) {
        (0..n).map(|_|{
            let rating = Rating {
                id: Uuid::new_v4().simple().to_string(),
                created: 0,
                title: "".into(),
                value: 2,
                context: RatingContext::Diversity,
                source: None
            };
            let triple = Triple {
                subject : ObjectId::Entry(id.into()),
                predicate : Relation::IsRatedWith,
                object : ObjectId::Rating(rating.id.clone()),
            };
            (rating,triple)
        })
        .unzip()
    }

    #[bench]
    fn bench_for_sorting_10_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings, triples) = create_entries_with_ratings_and_triples(10);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&ratings, &triples);
        });
    }

    #[bench]
    fn bench_for_sorting_100_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings, triples) = create_entries_with_ratings_and_triples(100);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&ratings, &triples);
        });
    }

    #[ignore]
    #[bench]
    fn bench_for_sorting_1000_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings, triples) = create_entries_with_ratings_and_triples(1000);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&ratings, &triples);
        });
    }

    #[ignore]
    #[bench]
    fn bench_for_sorting_2000_entries_by_rating(b: &mut Bencher) {
        let (entries, ratings, triples) = create_entries_with_ratings_and_triples(2000);
        b.iter(|| {
            let mut entries = entries.clone();
            entries.sort_by_avg_rating(&ratings, &triples);
        });
    }

    #[bench]
    fn bench_calc_avg_of_10_ratings_for_an_entry(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(10);
        b.iter(|| entry.avg_rating(&ratings, &triples));
    }

    #[bench]
    fn bench_calc_avg_of_100_ratings_for_an_entry(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(100);
        b.iter(|| entry.avg_rating(&ratings, &triples));
    }

    #[bench]
    fn bench_calc_avg_of_1000_ratings_for_an_entry(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(1000);
        b.iter(|| entry.avg_rating(&ratings, &triples));
    }

    #[bench]
    fn bench_calc_avg_of_10_ratings_for_a_rating_context(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(10);
        let entry_ratings = create_entry_ratings(&entry.id, &triples);
        b.iter(|| entry.avg_rating_for_context(&ratings, &entry_ratings, RatingContext::Diversity));
    }

    #[bench]
    fn bench_calc_avg_of_100_ratings_for_a_rating_context(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(100);
        let entry_ratings = create_entry_ratings(&entry.id, &triples);
        b.iter(|| entry.avg_rating_for_context(&ratings, &entry_ratings, RatingContext::Diversity));
    }

    #[bench]
    fn bench_calc_avg_of_1000_ratings_for_a_rating_context(b: &mut Bencher) {
        let (entry, ratings, triples) = create_entry_with_multiple_ratings_and_triples(1000);
        let entry_ratings = create_entry_ratings(&entry.id, &triples);
        b.iter(|| entry.avg_rating_for_context(&ratings, &entry_ratings, RatingContext::Diversity));
    }
}
