use brunch::{Bench, Benches};
use ofdb_core::rating::Rated;
use ofdb_entities::{builders::*, id::*, place::*, rating::*, time::*};

fn main() {
    let mut benches = Benches::default();

    let (entry, ratings) = create_place_with_multiple_ratings(1000);

    benches.push(
        Bench::new("Calculate avg. of 1000 ratings for an entry")
            .run(|| entry.avg_ratings(&ratings[..])),
    );
    benches.finish();
}

fn create_place_with_multiple_ratings(n: usize) -> (Place, Vec<Rating>) {
    let entry = Place::build().finish();
    let ratings = create_ratings_of_entry(entry.id.as_ref(), n);
    (entry, ratings)
}

fn create_ratings_of_entry(place_id: &str, n: usize) -> Vec<Rating> {
    (0..n)
        .map(|_| Rating {
            id: Id::new(),
            place_id: place_id.into(),
            created_at: Timestamp::now(),
            archived_at: None,
            title: "".into(),
            value: 2.into(),
            context: RatingContext::Diversity,
            source: None,
        })
        .collect()
}
