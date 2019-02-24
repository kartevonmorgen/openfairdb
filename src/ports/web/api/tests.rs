use super::{util::*, *};
use crate::{adapters::json, core::usecases as usecase, test::Bencher};
use pwhash::bcrypt;

pub mod prelude {
    use super::super::super::{rocket_instance, sqlite, tantivy};
    use rocket::{
        config::{Config, Environment},
        logger::LoggingLevel,
    };
    use std::fs;
    use uuid::Uuid;

    pub use crate::core::db::*;
    pub use rocket::{
        http::{ContentType, Cookie, Status},
        local::Client,
        response::Response,
    };

    pub fn setup() -> (Client, sqlite::ConnectionPool) {
        let cfg = Config::build(Environment::Development)
            .log_level(LoggingLevel::Debug)
            .finalize()
            .unwrap();
        let uuid = Uuid::new_v4().to_simple_ref().to_string();
        fs::create_dir_all("test-dbs").unwrap();
        let pool = sqlite::create_connection_pool(&format!("./test-dbs/{}", uuid)).unwrap();
        let search_engine = tantivy::create_search_engine_in_ram().unwrap();
        let rocket = rocket_instance(pool.clone(), search_engine, Some(cfg));
        let client = Client::new(rocket).unwrap();
        (client, pool)
    }

    pub fn setup2() -> (Client, sqlite::DbConn, tantivy::SearchEngine) {
        let cfg = Config::build(Environment::Development)
            .log_level(LoggingLevel::Debug)
            .finalize()
            .unwrap();
        let uuid = Uuid::new_v4().to_simple_ref().to_string();
        fs::create_dir_all("test-dbs").unwrap();
        let pool = sqlite::create_connection_pool(&format!("./test-dbs/{}", uuid)).unwrap();
        let search_engine = tantivy::create_search_engine_in_ram().unwrap();
        let rocket = rocket_instance(pool.clone(), search_engine.clone(), Some(cfg));
        let client = Client::new(rocket).unwrap();
        (client, sqlite::DbConn::new(pool), search_engine)
    }

    pub fn test_json(r: &Response) {
        assert_eq!(
            r.headers().get("Content-Type").collect::<Vec<_>>()[0],
            "application/json"
        );
    }
}

use self::prelude::*;

#[test]
fn create_entry() {
    let (client, db) = setup();
    db.get()
        .unwrap()
        .create_category_if_it_does_not_exist(&Category {
            id: "x".into(),
            created: 0,
            version: 0,
            name: "x".into(),
        })
        .unwrap();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":[]}"#);
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    let eid = db.get().unwrap().all_entries().unwrap()[0].id.clone();
    assert_eq!(body_str, format!("\"{}\"", eid));
}

#[test]
fn create_entry_with_reserved_tag() {
    let (client, db) = setup();
    db.get()
        .unwrap()
        .create_category_if_it_does_not_exist(&Category {
            id: "x".into(),
            created: 0,
            version: 0,
            name: "x".into(),
        })
        .unwrap();
    db.get()
        .unwrap()
        .create_org(Organization {
            id: "a".into(),
            name: "a".into(),
            owned_tags: vec!["a".into()],
            api_token: "a".into(),
        })
        .unwrap();
    let res = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["a"]}"#)
                    .dispatch();
    assert_eq!(res.status(), Status::Forbidden);
}

#[test]
fn create_entry_with_tag_duplicates() {
    let (client, db) = setup();
    db.get()
        .unwrap()
        .create_category_if_it_does_not_exist(&Category {
            id: "x".into(),
            created: 0,
            version: 0,
            name: "x".into(),
        })
        .unwrap();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["foo","foo"]}"#);
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    let eid = db.get().unwrap().all_entries().unwrap()[0].id.clone();
    assert_eq!(body_str, format!("\"{}\"", eid));
}

#[test]
fn create_entry_with_sharp_tag() {
    let (client, db) = setup();
    db.get()
        .unwrap()
        .create_category_if_it_does_not_exist(&Category {
            id: "x".into(),
            created: 0,
            version: 0,
            name: "x".into(),
        })
        .unwrap();
    let json = r##"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["foo","#bar"]}"##;
    let response = client
        .post("/entries")
        .header(ContentType::JSON)
        .body(json)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let tags = db.get().unwrap().all_entries().unwrap()[0].tags.clone();
    assert_eq!(tags, vec!["foo", "bar"]);
}

#[test]
fn update_entry_with_tag_duplicates() {
    let (client, db) = setup();
    db.get()
        .unwrap()
        .create_category_if_it_does_not_exist(&Category {
            id: "x".into(),
            created: 0,
            version: 0,
            name: "x".into(),
        })
        .unwrap();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["foo","foo"]}"#);
    let _res = req.dispatch();
    let e = db.get().unwrap().all_entries().unwrap()[0].clone();
    let mut json = String::new();
    json.push_str(&format!(
        "{{\"version\":{},\"id\":\"{}\"",
        e.version + 1,
        e.id
    ));
    json.push_str(r#","title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["bar","bar"]}"#);
    let url = format!("/entries/{}", e.id);
    let req = client.put(url).header(ContentType::JSON).body(json);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let e = db.get().unwrap().all_entries().unwrap()[0].clone();
    assert_eq!(e.tags, vec!["bar"]);
}

#[test]
fn get_one_entry() {
    let e = Entry::build()
        .id("get_one_entry_test")
        .title("some")
        .description("desc")
        .finish();

    let (client, db) = setup();
    db.get().unwrap().create_entry(e.clone()).unwrap();
    usecase::rate_entry(
        &mut *db.get().unwrap(),
        usecase::RateEntry {
            context: RatingContext::Humanity,
            value: 2,
            title: "title".into(),
            user: None,
            entry: "get_one_entry_test".into(),
            comment: "bla".into(),
            source: Some("blabla".into()),
        },
    )
    .unwrap();
    let req = client.get("/entries/get_one_entry_test");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<json::Entry> = serde_json::from_str(&body_str).unwrap();
    let rating = db.get().unwrap().all_ratings().unwrap()[0].clone();
    assert!(body_str.contains(&format!(r#""ratings":["{}"]"#, rating.id)));
    assert_eq!(
        entries[0],
        json::Entry::from_entry_with_ratings(e, vec![rating])
    );
}

#[test]
fn get_multiple_entries() {
    let one = Entry::build()
        .id("get_multiple_entry_test_one")
        .title("some")
        .description("desc")
        .finish();
    let two = Entry::build()
        .id("get_multiple_entry_test_two")
        .title("some")
        .description("desc")
        .finish();
    let (client, db) = setup();
    db.get().unwrap().create_entry(one.clone()).unwrap();
    db.get().unwrap().create_entry(two.clone()).unwrap();
    let req = client.get("/entries/get_multiple_entry_test_one,get_multiple_entry_test_two");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<json::Entry> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .any(|x| *x == json::Entry::from_entry_with_ratings(one.clone(), vec![])));
    assert!(entries
        .iter()
        .any(|x| *x == json::Entry::from_entry_with_ratings(two.clone(), vec![])));
}

fn default_new_entry() -> usecases::NewEntry {
    usecases::NewEntry {
        title: Default::default(),
        description: Default::default(),
        categories: Default::default(),
        email: None,
        telephone: None,
        lat: Default::default(),
        lng: Default::default(),
        street: None,
        zip: None,
        city: None,
        country: None,
        tags: Default::default(),
        homepage: None,
        license: "CC0-1.0".into(),
        image_url: None,
        image_link_url: None,
    }
}

fn new_entry_with_category(category: &str, latlng: f64) -> usecases::NewEntry {
    usecases::NewEntry {
        categories: vec![category.into()],
        lat: latlng,
        lng: latlng,
        ..default_new_entry()
    }
}

#[test]
fn search_with_categories() {
    let entries = vec![
        new_entry_with_category("foo", 1.0),
        new_entry_with_category("foo", 2.0),
        new_entry_with_category("bar", 3.0),
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_category_if_it_does_not_exist(&Category {
        id: "foo".into(),
        created: 0,
        version: 0,
        name: "foo".into(),
    })
    .unwrap();
    db.create_category_if_it_does_not_exist(&Category {
        id: "bar".into(),
        created: 0,
        version: 0,
        name: "bar".into(),
    })
    .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&categories=foo&limit=2");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&categories=bar&limit=1");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&categories=foo,bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

fn new_entry_with_text(title: &str, description: &str, latlng: f64) -> usecases::NewEntry {
    usecases::NewEntry {
        title: title.into(),
        description: description.into(),
        lat: latlng,
        lng: latlng,
        ..default_new_entry()
    }
}

#[test]
fn search_with_text() {
    let entries = vec![
        new_entry_with_text("Foo", "bla", 1.0),
        new_entry_with_text("bar", "foo", 2.0),
        new_entry_with_text("baZ", "blub", 3.0),
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();
    let req = client.get("/search?bbox=-10,-10,10,10&text=Foo");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[ignore]
#[bench]
fn bench_search_in_10_000_rated_entries(b: &mut Bencher) {
    let (entries, ratings) = crate::core::util::sort::tests::create_entries_with_ratings(10_000);
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    for e in entries {
        conn.create_entry(e).unwrap();
    }
    for r in ratings {
        conn.create_rating(r).unwrap();
    }
    b.iter(|| client.get("/search?bbox=-10,-10,10,10").dispatch());
}

#[test]
fn search_with_tags() {
    let entries = vec![
        usecases::NewEntry {
            categories: vec!["foo".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            categories: vec!["foo".to_string()],
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            categories: vec!["foo".to_string()],
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_category_if_it_does_not_exist(&Category {
        id: "foo".into(),
        created: 0,
        version: 0,
        name: "foo".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    })
    .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();
    let req = client.get("/search?bbox=-10,-10,10,10&tags=bla-blubb");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!(
        "\"visible\":[{{\"id\":\"{}\",\"lat\":0.0,\"lng\":0.0}}]",
        entry_ids[1]
    )));

    let req = client.get("/search?bbox=-10,-10,10,10&tags=foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[test]
fn search_with_uppercase_tags() {
    let entries = vec![
        usecases::NewEntry {
            tags: vec!["fOO".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["fooo".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["Foo".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "foo".into() })
        .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "bar".into() })
        .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "baz".into() })
        .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();
    let req = client.get("/search?bbox=-10,-10,10,10&tags=Foo");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[test]
fn search_with_hashtag() {
    let entries = vec![
        usecases::NewEntry {
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    })
    .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[test]
fn search_with_two_hashtags() {
    let entries = vec![
        usecases::NewEntry {
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    })
    .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();

    // exact match
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23bla-blubb%20%23foo-bar&limit=1");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));

    // partial matches
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23bla-blubb%20%23foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[test]
fn search_with_commata() {
    let entries = vec![
        usecases::NewEntry {
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["eins".to_string(), "zwei".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["eins".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            tags: vec!["eins".to_string(), "zwei".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "eins".into() })
        .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "zwei".into() })
        .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();

    // exact matches
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23eins%2C%20%23zwei&limit=2");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[3])));

    // partial matches
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23eins%2C%20%23zwei");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[3])));
}

#[test]
fn search_without_specifying_hashtag_symbol() {
    let entries = vec![
        usecases::NewEntry {
            title: "foo".into(),
            ..default_new_entry()
        },
        usecases::NewEntry {
            title: "foo".into(),
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewEntry {
            title: "foo".into(),
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    })
    .unwrap();
    let entry_ids: Vec<_> = entries
        .into_iter()
        .map(|e| usecases::create_new_entry(&mut *db, Some(&mut search_engine), e).unwrap())
        .collect();

    let mut response = client.get("/search?bbox=-10,-10,10,10&text=foo").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=%23foo")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));

    // Text "foo-bar" is tokenized into "foo" and "bar"
    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=foo-bar")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=%23foo-bar")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=%23bla-blubb")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));

    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=bla-blubb")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", entry_ids[2])));
}

#[test]
fn extract_ids_test() {
    assert_eq!(extract_ids("abc"), vec!["abc"]);
    assert_eq!(extract_ids("a, b,c"), vec!["a", "b", "c"]);
    assert_eq!(extract_ids("\t").len(), 0);
    assert_eq!(extract_ids("abc, ,d,"), vec!["abc", "d"]);
}

#[test]
fn extract_single_hash_tag_from_text() {
    assert_eq!(extract_hash_tags("none").len(), 0);
    assert_eq!(extract_hash_tags("#").len(), 0);
    assert_eq!(extract_hash_tags("foo #bar none"), vec!["bar".to_string()]);
    assert_eq!(extract_hash_tags("foo #bar,none"), vec!["bar".to_string()]);
    assert_eq!(extract_hash_tags("foo#bar,none"), vec!["bar".to_string()]);
    assert_eq!(
        extract_hash_tags("foo#bar none#baz"),
        vec!["bar".to_string(), "baz".to_string()]
    );
    assert_eq!(
        extract_hash_tags("#bar#baz"),
        vec!["bar".to_string(), "baz".to_string()]
    );
    assert_eq!(
        extract_hash_tags("#a-long-tag#baz"),
        vec!["a-long-tag".to_string(), "baz".to_string()]
    );
    assert_eq!(extract_hash_tags("#-").len(), 0);
    assert_eq!(extract_hash_tags("#tag-"), vec!["tag".to_string()]);
}

#[test]
fn remove_hash_tag_from_text() {
    assert_eq!(remove_hash_tags("some #tag"), "some");
    assert_eq!(remove_hash_tags("some#tag"), "some");
    assert_eq!(remove_hash_tags("#tag"), "");
    assert_eq!(remove_hash_tags("some #text with #tags"), "some with");
}

#[test]
fn create_new_user() {
    let (client, db) = setup();
    let req = client
        .post("/users")
        .header(ContentType::JSON)
        .body(r#"{"username":"foo","email":"foo@bar.com","password":"bar"}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let u = db.get().unwrap().get_user("foo").unwrap();
    assert_eq!(u.username, "foo");
    assert!(bcrypt::verify("bar", &u.password));
    test_json(&response);
}

#[test]
fn create_rating() {
    let (client, db) = setup();
    let entries = vec![Entry::build().id("foo").finish()];
    let mut conn = db.get().unwrap();
    for e in entries {
        conn.create_entry(e).unwrap();
    }
    let req = client.post("/ratings")
        .header(ContentType::JSON)
        .body(r#"{"value": 1,"context":"fairness","entry":"foo","comment":"test", "title":"idontcare", "source":"source..."}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(db.get().unwrap().all_ratings().unwrap()[0].value, 1);
    test_json(&response);
}

#[test]
fn get_one_rating() {
    let e = Entry::build().id("foo").finish();
    let (client, db) = setup();
    db.get().unwrap().create_entry(e).unwrap();
    usecase::rate_entry(
        &mut *db.get().unwrap(),
        usecase::RateEntry {
            context: RatingContext::Humanity,
            value: 2,
            user: None,
            title: "title".into(),
            entry: "foo".into(),
            comment: "bla".into(),
            source: Some("blabla".into()),
        },
    )
    .unwrap();
    let rid = db.get().unwrap().all_ratings().unwrap()[0].id.clone();
    let req = client.get(format!("/ratings/{}", rid));
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let ratings: Vec<json::Rating> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(ratings[0].id, rid);
    assert_eq!(ratings[0].comments.len(), 1);
}

#[test]
fn ratings_with_and_without_source() {
    let e1 = Entry::build().id("foo").finish();
    let e2 = Entry::build().id("bar").finish();
    let (client, db) = setup();
    db.get().unwrap().create_entry(e1).unwrap();
    db.get().unwrap().create_entry(e2).unwrap();
    usecase::rate_entry(
        &mut *db.get().unwrap(),
        usecase::RateEntry {
            context: RatingContext::Humanity,
            value: 2,
            user: None,
            title: "title".into(),
            entry: "foo".into(),
            comment: "bla".into(),
            source: Some("blabla blabla".into()),
        },
    )
    .unwrap();
    usecase::rate_entry(
        &mut *db.get().unwrap(),
        usecase::RateEntry {
            context: RatingContext::Humanity,
            value: 2,
            user: None,
            title: "title".into(),
            entry: "bar".into(),
            comment: "bla".into(),
            source: Some("blabla blabla".into()),
        },
    )
    .unwrap();

    let rid = db.get().unwrap().all_ratings().unwrap()[0].id.clone();
    let req = client.get(format!("/ratings/{}", rid));
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let ratings: Vec<json::Rating> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(ratings[0].id, rid);
    assert_eq!(ratings[0].comments.len(), 1);
}

fn user_id_cookie(response: &Response) -> Option<Cookie<'static>> {
    let cookie = response
        .headers()
        .get("Set-Cookie")
        .filter(|v| v.starts_with("user_id"))
        .nth(0)
        .and_then(|val| Cookie::parse_encoded(val).ok());

    cookie.map(|c| c.into_owned())
}

#[test]
fn post_user() {
    let (client, _) = setup();
    let req1 = client.post("/users").header(ContentType::JSON).body(
        r#"{"username": "foo12341234", "email": "123412341234foo@bar.de", "password": "bar"}"#,
    );
    let response1 = req1.dispatch();
    assert_eq!(response1.status(), Status::Ok);

    let req2 = client.post("/users").header(ContentType::JSON).body(
        r#"{"username": "baz14234134", "email": "123412341234baz@bar.de", "password": "bar"}"#,
    );
    let response2 = req2.dispatch();
    assert_eq!(response2.status(), Status::Ok);
}

#[test]
fn login_with_invalid_credentials() {
    let (client, _) = setup();
    let req = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#);
    let response = req.dispatch();
    assert!(!response
        .headers()
        .iter()
        .any(|h| h.name.as_str() == "Set-Cookie"));
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn login_with_valid_credentials() {
    let (client, db) = setup();
    let users = vec![User {
        id: "123".into(),
        username: "foo".into(),
        password: bcrypt::hash("bar").unwrap(),
        email: "foo@bar".into(),
        email_confirmed: true,
        role: Role::Guest,
    }];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let cookie = user_id_cookie(&response).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().len() > 25);
}

#[test]
fn login_logout_succeeds() {
    let (client, db) = setup();
    let users = vec![User {
        id: "123".into(),
        username: "foo".into(),
        password: bcrypt::hash("bar").unwrap(),
        email: "foo@bar".into(),
        email_confirmed: true,
        role: Role::Guest,
    }];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }

    // Login
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let cookie = user_id_cookie(&response).expect("login cookie");

    // Logout
    let response = client
        .post("/logout")
        .header(ContentType::JSON)
        .cookie(cookie)
        .dispatch();
    let cookie = user_id_cookie(&response).expect("logout cookie");
    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().is_empty());
}

#[test]
fn get_user() {
    let (client, db) = setup();
    let users = vec![
        User {
            id: "123".into(),
            username: "a".into(),
            password: bcrypt::hash("a").unwrap(),
            email: "a@bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        },
        User {
            id: "123".into(),
            username: "b".into(),
            password: bcrypt::hash("b").unwrap(),
            email: "b@bar".into(),
            email_confirmed: true,
            role: Role::Guest,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "a", "password": "a"}"#)
        .dispatch();

    let cookie = user_id_cookie(&response).unwrap();

    let response = client
        .get("/users/b")
        .header(ContentType::JSON)
        .cookie(cookie.clone())
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);

    let mut response = client
        .get("/users/a")
        .header(ContentType::JSON)
        .cookie(cookie)
        .dispatch();

    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(body_str, r#"{"username":"a","email":"a@bar"}"#);
    test_json(&response);
}

#[test]
fn confirm_email_address() {
    let (client, db) = setup();
    let users = vec![User {
        id: "123".into(),
        username: "foo".into(),
        password: bcrypt::hash("bar").unwrap(),
        email: "a@bar.de".into(),
        email_confirmed: false,
        role: Role::Guest,
    }];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }

    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
    assert_eq!(
        db.get().unwrap().all_users().unwrap()[0].email_confirmed,
        false
    );

    let response = client
        .post("/confirm-email-address")
        .header(ContentType::JSON)
        .body(r#"{"u_id": "123"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        db.get().unwrap().all_users().unwrap()[0].email_confirmed,
        true
    );

    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let cookie: Cookie = response
        .headers()
        .iter()
        .filter(|h| h.name == "Set-Cookie")
        .filter(|h| h.value.contains("user_id="))
        .nth(0)
        .unwrap()
        .value
        .parse()
        .unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().len() > 25);
}

//TODO: make it pass
#[ignore]
#[test]
fn send_confirmation_email() {
    let (client, db) = setup();
    let users = vec![User {
        id: "123".into(),
        username: "foo".into(),
        password: bcrypt::hash("bar").unwrap(),
        email: "a@bar.de".into(),
        email_confirmed: false,
        role: Role::Guest,
    }];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }

    let response = client
        .post("/send-confirmation-email")
        .header(ContentType::JSON)
        .body(r#""foo""#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn subscribe_to_bbox() {
    let (client, db) = setup();
    let users = vec![User {
        id: "123".into(),
        username: "foo".into(),
        password: bcrypt::hash("bar").unwrap(),
        email: "foo@bar".into(),
        email_confirmed: true,
        role: Role::Guest,
    }];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(u).unwrap();
    }
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let cookie = user_id_cookie(&response).unwrap();
    let response = client
        .post("/subscribe-to-bbox")
        .header(ContentType::JSON)
        .cookie(cookie)
        .body(r#"[{"lat":-10.0,"lng":-10.0},{"lat":10.0,"lng":10.0}]"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn openapi() {
    let (client, _) = setup();
    let req = client.get("/server/api.yaml");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get("Content-Type").collect::<Vec<_>>()[0],
        "text/yaml"
    );
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("openapi:"))
}

#[test]
fn export_csv() {
    let mut entries = vec![
        Entry::build()
            .id("entry1")
            .version(3)
            .title("title1")
            .description("desc1")
            .lat(0.1)
            .lng(0.2)
            .categories(vec![
                "2cd00bebec0c48ba9db761da48678134",
                "77b3c33a92554bcf8e8c2c86cedd6f6f",
            ])
            .tags(vec!["bli", "bla"])
            .license(Some("license1"))
            .finish(),
        Entry::build()
            .id("entry2")
            .categories(vec!["2cd00bebec0c48ba9db761da48678134"])
            .lat(0.0)
            .lng(0.0)
            .finish(),
        Entry::build()
            .id("entry3")
            .categories(vec!["77b3c33a92554bcf8e8c2c86cedd6f6f"])
            .lat(2.0)
            .lng(2.0)
            .finish(),
    ];
    entries[0].location.address = Some(Address::build().street("street1").finish());
    entries[0].osm_node = Some(1);
    entries[0].created = 2;
    entries[0].location.address = Some(
        Address::build()
            .street("street1")
            .zip("zip1")
            .city("city1")
            .country("country1")
            .finish(),
    );
    entries[0].homepage = Some("homepage1".to_string());

    let (client, db, mut search_engine) = setup2();
    let mut db = db.read_write().unwrap();

    db.create_category_if_it_does_not_exist(&Category {
        id: "2cd00bebec0c48ba9db761da48678134".into(),
        created: 0,
        version: 0,
        name: "cat1".into(),
    })
    .unwrap();
    db.create_category_if_it_does_not_exist(&Category {
        id: "77b3c33a92554bcf8e8c2c86cedd6f6f".into(),
        created: 0,
        version: 0,
        name: "cat2".into(),
    })
    .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "bli".into() })
        .unwrap();
    db.create_tag_if_it_does_not_exist(&Tag { id: "bla".into() })
        .unwrap();
    for e in entries {
        search_engine.add_or_update_entry(&e).unwrap();
        db.create_entry(e).unwrap();
    }
    search_engine.flush().unwrap();
    let diversity = RatingContext::Diversity;
    db.create_rating(Rating {
        id: "123".into(),
        entry_id: "entry1".into(),
        created: 123,
        title: "rating1".into(),
        value: 2,
        context: diversity.clone(),
        source: None,
    })
    .unwrap();
    db.create_rating(Rating {
        id: "345".into(),
        entry_id: "entry1".into(),
        created: 123,
        title: "rating2".into(),
        value: 4,
        context: diversity,
        source: None,
    })
    .unwrap();

    super::super::calculate_all_ratings(&*db).unwrap();

    let req = client.get("/export/entries.csv?bbox=-1,-1,1,1");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "text/csv; charset=utf-8"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str, "id,osm_node,created,version,title,description,lat,lng,street,zip,city,country,homepage,categories,tags,license,avg_rating\n\
        entry1,1,2,3,title1,desc1,0.1,0.2,street1,zip1,city1,country1,homepage1,\"cat1,cat2\",\"bla,bli\",license1,0.5\n\
        entry2,,0,0,,,0,0,,,,,,cat1,,,0\n");
}
