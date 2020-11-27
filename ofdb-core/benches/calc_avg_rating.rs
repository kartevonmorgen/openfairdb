use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ofdb_core::rating::Rated;
use ofdb_entities::{builders::*, id::*, place::*, rating::*, time::*};

fn calc_avg_of_1000_ratings_for_an_entry(c: &mut Criterion) {
    let (entry, ratings) = create_place_with_multiple_ratings(black_box(1000));
    c.bench_function("avg_ratings 1000", |b| {
        b.iter(|| entry.avg_ratings(&ratings[..]))
    });
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

criterion_group!(benches, calc_avg_of_1000_ratings_for_an_entry);
criterion_main!(benches);
