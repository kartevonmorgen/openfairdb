use std::fmt::Write as _;

use ofdb_core::rating::Rated;

use super::*;
use crate::{adapters::json, core::usecases};

pub mod prelude {

    use crate::web::{self, api, sqlite, tantivy};
    use ofdb_core::gateways::notify::NotificationGateway;
    use std::collections::HashSet;

    pub use crate::web::{
        api::captcha::tests::get_valid_captcha_cookie as get_captcha_cookie,
        tests::prelude::{LocalResponse as Response, *},
        Cfg,
    };

    pub fn setup() -> (Client, sqlite::Connections) {
        setup_with_cfg(Cfg {
            accepted_licenses: default_accepted_licenses(),
            protect_with_captcha: false,
        })
    }

    pub fn setup_with_cfg(cfg: Cfg) -> (Client, sqlite::Connections) {
        let (client, conn, _) = web::tests::setup_with_cfg(vec![("/", api::routes())], cfg);
        (client, conn)
    }

    pub fn setup2() -> (
        Client,
        sqlite::Connections,
        tantivy::SearchEngine,
        impl NotificationGateway,
    ) {
        let (client, connections, search_engine) = web::tests::setup(vec![("/", api::routes())]);
        (client, connections, search_engine, DummyNotifyGW {})
    }

    pub fn test_json(r: &Response) {
        assert_eq!(
            r.headers().get("Content-Type").collect::<Vec<_>>()[0],
            "application/json"
        );
    }

    pub use super::cookie_from_response;

    pub fn default_accepted_licenses() -> std::collections::HashSet<String> {
        let mut accepted_licenses = HashSet::new();
        accepted_licenses.insert("CC0-1.0".into());
        accepted_licenses.insert("ODbL-1.0".into());
        accepted_licenses
    }

    pub fn create_place(client: &Client) -> ofdb_entities::id::Id {
        let body_string = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"license":"CC0-1.0","tags":["foo"]}"#).dispatch().into_string().unwrap();
        serde_json::from_str::<String>(&body_string).unwrap().into()
    }
}

use self::prelude::*;

#[test]
fn create_a_new_place() {
    let (client, db) = setup();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":[]}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    let eid = db.exclusive().unwrap().all_places().unwrap()[0]
        .0
        .id
        .clone();
    assert_eq!(body_str, format!("\"{}\"", eid));
}

#[test]
fn create_place_with_reserved_tag() {
    let (client, db) = setup();
    db.exclusive()
        .unwrap()
        .create_org(Organization {
            id: "a".into(),
            name: "a".into(),
            moderated_tags: vec!["a".into()],
            api_token: "a".into(),
        })
        .unwrap();
    let cookie = get_captcha_cookie(&client).unwrap();
    let res = client.post("/entries")
                    .header(ContentType::JSON)
                    .cookie(cookie)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["a"]}"#)
                    .dispatch();
    assert_eq!(res.status(), Status::Forbidden);
}

#[test]
fn create_place_with_tag_duplicates() {
    let (client, db) = setup();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["foo","foo"]}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    let eid = db.exclusive().unwrap().all_places().unwrap()[0]
        .0
        .id
        .clone();
    assert_eq!(body_str, format!("\"{}\"", eid));
}

#[test]
fn create_place_with_sharp_tag_and_custom_link() {
    let (client, db) = setup();
    let json = r##"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["foo","#bar","#foo&bar","foo#bar"],"links":[{"url":"example.com","title":"Auto-completed URL"}]}"##;
    let response = client
        .post("/entries")
        .header(ContentType::JSON)
        .body(json)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let (place, _) = db
        .shared()
        .unwrap()
        .all_places()
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(&place.tags[..], &["bar", "foo", "foo&bar", "foobar"]);
    // The "https://www." prefix should be added implicitly by auto-completion!
    assert_eq!(
        &place.links.unwrap().custom[..],
        &[CustomLink {
            url: "https://www.example.com".parse().unwrap(),
            title: Some("Auto-completed URL".to_string()),
            description: None
        }]
    );
}

#[test]
fn update_place_with_tag_duplicates() {
    let (client, db) = setup();
    let req = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"ODbL-1.0","tags":["foo","foo"]}"#);
    let _res = req.dispatch();
    let (place, _) = db.exclusive().unwrap().all_places().unwrap()[0].clone();
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"version\":{},\"id\":\"{}\"",
        u64::from(place.revision.next()),
        place.id
    )
    .unwrap();
    json.push_str(r#","title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["bar","bar"]}"#);
    let url = format!("/entries/{}", place.id);
    let req = client.put(url).header(ContentType::JSON).body(json);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let (e, _) = db.exclusive().unwrap().all_places().unwrap()[0].clone();
    assert_eq!(e.tags, vec!["bar"]);
}

#[test]
fn get_one_entry() {
    let e = Place::build()
        .id("get_one_entry_test")
        .title("some")
        .description("desc")
        .finish();

    let (client, connections, mut search_engine, _) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_or_update_place(e.clone())
        .unwrap();
    flows::create_rating(
        &connections,
        &mut *search_engine,
        usecases::NewPlaceRating {
            context: RatingContext::Humanity,
            value: RatingValue::from(2),
            title: "title".into(),
            user: None,
            entry: "get_one_entry_test".into(),
            comment: "bla".into(),
            source: Some("blabla".into()),
        },
    )
    .unwrap();
    let req = client.get("/entries/get_one_entry_test");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str.as_str().chars().next().unwrap(), '[');
    let entries: Vec<json::Entry> = serde_json::from_str(&body_str).unwrap();
    let rating = connections
        .shared()
        .unwrap()
        .load_ratings_of_place("get_one_entry_test")
        .unwrap()[0]
        .clone();
    assert!(body_str.contains(&format!(r#""ratings":["{}"]"#, rating.id)));
    assert_eq!(
        entries[0],
        json::entry_from_place_with_ratings(e, vec![rating])
    );
}

#[test]
fn get_multiple_places() {
    let one = Place::build()
        .id("get_multiple_entry_test_one")
        .title("some")
        .description("desc")
        .finish();
    let two = Place::build()
        .id("get_multiple_entry_test_two")
        .title("some")
        .description("desc")
        .finish();
    let (client, db) = setup();
    db.exclusive()
        .unwrap()
        .create_or_update_place(one.clone())
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_or_update_place(two.clone())
        .unwrap();
    let req = client.get("/entries/get_multiple_entry_test_one,get_multiple_entry_test_two");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str.as_str().chars().next().unwrap(), '[');
    let entries: Vec<json::Entry> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .any(|x| *x == json::entry_from_place_with_ratings(one.clone(), vec![])));
    assert!(entries
        .iter()
        .any(|x| *x == json::entry_from_place_with_ratings(two.clone(), vec![])));
}

fn default_new_entry() -> usecases::NewPlace {
    usecases::NewPlace {
        title: Default::default(),
        description: Default::default(),
        categories: Default::default(),
        contact_name: None,
        email: None,
        telephone: None,
        lat: Default::default(),
        lng: Default::default(),
        street: None,
        zip: None,
        city: None,
        country: None,
        state: None,
        tags: Default::default(),
        homepage: None,
        opening_hours: None,
        founded_on: None,
        license: "CC0-1.0".into(),
        image_url: None,
        image_link_url: None,
        custom_links: vec![],
    }
}

fn new_entry_with_category(category: &str, lat: f64, lng: f64) -> usecases::NewPlace {
    usecases::NewPlace {
        categories: vec![category.into()],
        lat,
        lng,
        ..default_new_entry()
    }
}

#[test]
fn search_with_categories_and_bbox() {
    let entries = vec![
        new_entry_with_category(Category::ID_NON_PROFIT, 1.0, 1.0),
        new_entry_with_category(Category::ID_NON_PROFIT, 2.0, 2.0),
        new_entry_with_category(Category::ID_COMMERCIAL, 3.0, 3.0),
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get(format!(
        "/search?bbox=-10,-10,10,10&categories={}",
        Category::ID_NON_PROFIT
    ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get(format!(
        "/search?bbox=1.8,0.5,3.0,3.0&categories={}",
        Category::ID_NON_PROFIT
    ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get(format!(
        "/search?bbox=-10,-10,10,10&categories={}",
        Category::ID_COMMERCIAL
    ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get(format!(
        "/search?bbox=-10,-10,10,10&categories={},{}",
        Category::ID_NON_PROFIT,
        Category::ID_COMMERCIAL
    ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get(format!(
        "/search?bbox=0.9,0.5,2.5,2.0&categories={},{}",
        Category::ID_NON_PROFIT,
        Category::ID_COMMERCIAL
    ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
}

fn new_entry_with_text(title: &str, description: &str, lat: f64, lng: f64) -> usecases::NewPlace {
    usecases::NewPlace {
        title: title.into(),
        description: description.into(),
        lat,
        lng,
        ..default_new_entry()
    }
}

#[test]
fn search_with_text() {
    let entries = vec![
        new_entry_with_text("Foo", "bla", 1.0, 1.0),
        new_entry_with_text("bar", "foo", 2.0, 2.0),
        new_entry_with_text("baZ", "blub", 3.0, 3.0),
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    // Search case insensitive "Foo" and "foo"
    // Limit is required, because all entries match the query and their
    // rating is equal. The match score is currently not considered when
    // ordering the results!
    let req = client.get("/search?bbox=-10,-10,10,10&text=Foo&limit=2");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    // Search case insensitive "Foo" and "foo" with bbox
    let req = client.get("/search?bbox=1.8,0.5,3.0,3.0&text=Foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    // Search with whitespace
    let req = client.get("/search?bbox=-10,-10,10,10&text=blub%20foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    // Search with whitespace and bbox
    let req = client.get("/search?bbox=0.9,0.5,2.5,2.0&text=blub%20foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    // Search with punctuation
    // TODO: Ignore punctuation in query text and make this test pass
    // See also: https://github.com/slowtec/openfairdb/issues/82
    /*
    let req = client.get("/search?bbox=-10,-10,10,10&text=blub,Foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
    */
}

#[test]
fn search_partial_text() {
    let entries = vec![
        new_entry_with_text("Foo", "bla", 1.0, 1.0),
        new_entry_with_text("bar", "foo", 2.0, 2.0),
        new_entry_with_text("baZ", "blub", 3.0, 3.0),
        new_entry_with_text("foo-bar-BaZ", "blub-blub", 1.0, 1.0),
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&text=bar-baz");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[3])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=blub-");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2]))); // trailing '-' is ignored by Tantivy!
    assert!(body_str.contains(&format!("\"{}\"", place_ids[3])));
}

#[test]
fn search_with_text_terms_inclusive_exclusive() {
    let entries = vec![
        new_entry_with_text("foo", "bar", 1.0, 1.0),
        new_entry_with_text("fOO", "baz", 2.0, 2.0),
        new_entry_with_text("baZ", "Bar", 3.0, 3.0),
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&text=+Foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=+Foo%20-BAZ");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=+Foo%20+BAZ&limit=1");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=+Foo%20+BAZ%20-bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=+bAz%20+BAr&limit=1");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=-foo%20+bAz%20+BAr");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=-foo%20+bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let req = client.get("/search?bbox=-10,-10,10,10&text=+foo+bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
}

fn new_entry_with_city(city: &str, latlng: f64) -> usecases::NewPlace {
    usecases::NewPlace {
        city: Some(city.into()),
        lat: latlng,
        lng: latlng,
        ..default_new_entry()
    }
}

#[test]
fn search_with_city() {
    let entries = vec![
        new_entry_with_city("Stuttgart", 1.0),
        new_entry_with_city("Mannheim", 2.0),
        new_entry_with_city("Stuttgart-Möhringen", 3.0),
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();
    search_engine.flush_index().unwrap();

    // Limit is required, because all entries match the query and their
    // rating is equal. The match score is currently not considered when
    // ordering the results!
    let req = client.get("/search?bbox=-10,-10,10,10&text=stuttgart&limit=2");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_tags() {
    let entries = vec![
        usecases::NewPlace {
            categories: vec![Category::ID_NON_PROFIT.to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            categories: vec![Category::ID_NON_PROFIT.to_string()],
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            categories: vec![Category::ID_NON_PROFIT.to_string()],
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: Category::TAG_NON_PROFIT.into(),
        })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "foo-bar".into(),
        })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "bla-blubb".into(),
        })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&tags=bla-blubb");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!(
        "\"visible\":[{{\"id\":\"{}\",\"status\":\"created\",\"lat\":0.0,\"lng\":0.0,\"title\":\"\",\"description\":\"\",\"categories\":[\"{}\"],\"tags\":[\"bla-blubb\",\"foo-bar\"],\"ratings\":{{\"total\":0.0,\"diversity\":0.0,\"fairness\":0.0,\"humanity\":0.0,\"renewable\":0.0,\"solidarity\":0.0,\"transparency\":0.0}}}}]",
        place_ids[1],
        Category::ID_NON_PROFIT,
    )));

    let req = client.get("/search?bbox=-10,-10,10,10&tags=foo-bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_uppercase_tags() {
    let entries = vec![
        usecases::NewPlace {
            tags: vec!["fOO".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["fooo".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["Foo".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "foo".into() })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "bar".into() })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "baz".into() })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&tags=Foo");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_hashtag() {
    let entries = vec![
        usecases::NewPlace {
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "bla-blubb".into(),
        })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "foo-bar".into(),
        })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&text=%23foo-bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_two_hashtags() {
    let entries = vec![
        usecases::NewPlace {
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "bla-blubb".into(),
        })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "foo-bar".into(),
        })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
        })
        .collect();

    let req = client.get("/search?bbox=-10,-10,10,10&text=%23bla-blubb%20%23foo-bar");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_commata() {
    let entries = vec![
        usecases::NewPlace {
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["eins".to_string(), "zwei".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            tags: vec!["eins".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "eins".to_string(),
            tags: vec!["zwei".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "eins".to_string(),
            description: "zwei".to_string(),
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "eins".into() })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "zwei".into() })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    // With hashtag symbol '#' -> all hashtags are mandatory
    // #eins + #zwei
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23eins%2C%20%23zwei");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[3])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[4])));

    // Without hashtag symbol '#' -> tags are optional
    // eins + #zwei
    let req = client.get("/search?bbox=-10,-10,10,10&text=eins%2C%20%23zwei");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[3])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[4])));

    // Without hashtag symbol '#' -> tags are optional
    // eins + zwei
    let req = client.get("/search?bbox=-10,-10,10,10&text=eins%2C%20zwei");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[3])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[4])));
}

#[test]
fn search_without_specifying_hashtag_symbol() {
    let entries = vec![
        usecases::NewPlace {
            title: "foo".into(),
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "foo".into(),
            tags: vec!["bla-blubb".to_string(), "foo-bar".to_string()],
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "foo".into(),
            tags: vec!["foo-bar".to_string()],
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "bla-blubb".into(),
        })
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: "foo-bar".into(),
        })
        .unwrap();
    let place_ids: Vec<_> = entries
        .into_iter()
        .map(|e| {
            flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                e,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string()
        })
        .collect();

    let response = client.get("/search?bbox=-10,-10,10,10&text=foo").dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let response = client
        .get("/search?bbox=-10,-10,10,10&text=%23foo")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    // Text "foo-bar" is tokenized into "foo" and "bar"
    let response = client
        .get("/search?bbox=-10,-10,10,10&text=foo-bar")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let response = client
        .get("/search?bbox=-10,-10,10,10&text=%23foo-bar")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[2])));

    let response = client
        .get("/search?bbox=-10,-10,10,10&text=%23bla-blubb")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));

    let response = client
        .get("/search?bbox=-10,-10,10,10&text=bla-blubb")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[0])));
    assert!(body_str.contains(&format!("\"{}\"", place_ids[1])));
    assert!(!body_str.contains(&format!("\"{}\"", place_ids[2])));
}

#[test]
fn search_with_status() {
    let places = vec![
        usecases::NewPlace {
            title: "created".into(),
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "confirmed".into(),
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "rejected".into(),
            ..default_new_entry()
        },
        usecases::NewPlace {
            title: "archived".into(),
            ..default_new_entry()
        },
    ];
    let (client, connections, mut search_engine, notify) = setup2();

    let places: Vec<_> = places
        .into_iter()
        .map(|p| {
            let status = p.title.clone();
            let id = flows::create_place(
                &connections,
                &mut *search_engine,
                &notify,
                p,
                None,
                None,
                &default_accepted_licenses(),
            )
            .unwrap()
            .id
            .to_string();
            (id, status)
        })
        .collect();

    let user = User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Admin,
    };
    connections.exclusive().unwrap().create_user(&user).unwrap();
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    for (id, status) in &places {
        let req = client
            .post(format!("/places/{}/review", id))
            .header(ContentType::JSON)
            .body(format!(
                "{{\"status\":\"{}\",\"comment\":\"{}\"}}",
                status, id
            ));
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    // All visible = created + confirmed
    let req = client.get("/search?bbox=-10,-10,10,10");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains(&format!("\"{}\"", places[0].0)));
    assert!(body_str.contains(&format!("\"{}\"", places[0].1)));
    assert!(body_str.contains(&format!("\"{}\"", places[1].0)));
    assert!(body_str.contains(&format!("\"{}\"", places[1].1)));
    assert!(!body_str.contains(&format!("\"{}\"", places[2].0)));
    assert!(!body_str.contains(&format!("\"{}\"", places[2].1)));
    assert!(!body_str.contains(&format!("\"{}\"", places[3].0)));
    assert!(!body_str.contains(&format!("\"{}\"", places[3].1)));

    // Single status search
    for (id, status) in &places {
        let req = client.get(format!("/search?bbox=-10,-10,10,10&status={}", status));
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_str = response.into_string().unwrap();
        assert!(body_str.contains(&format!("\"{}\"", id)));
        assert!(body_str.contains(&format!("\"{}\"", status)));
        for (other_id, other_status) in places.iter().filter(|(other_id, _)| other_id != id) {
            assert!(!body_str.contains(&format!("\"{}\"", other_id)));
            assert!(!body_str.contains(&format!("\"{}\"", other_status)));
        }
    }
}

#[test]
fn create_new_user() {
    let (client, db) = setup();
    let req = client
        .post("/users")
        .header(ContentType::JSON)
        .body(r#"{"email":"foo@bar.com","password":"foo bar"}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let u = db
        .exclusive()
        .unwrap()
        .get_user_by_email(&"foo@bar.com".parse().unwrap())
        .unwrap();
    assert_eq!(u.email.as_str(), "foo@bar.com");
    assert!(u.password.verify("foo bar"));
    test_json(&response);
}

#[test]
fn create_rating() {
    let (client, connections, _, _) = setup2();
    let entries = vec![Place::build().id("foo").finish()];
    for e in entries {
        connections
            .exclusive()
            .unwrap()
            .create_or_update_place(e)
            .unwrap();
    }
    let req = client.post("/ratings")
        .header(ContentType::JSON)
        .body(r#"{"value": 1,"context":"fairness","entry":"foo","comment":"test", "title":"idontcare", "source":"source..."}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        connections
            .shared()
            .unwrap()
            .load_ratings_of_place("foo")
            .unwrap()[0]
            .value,
        RatingValue::from(1)
    );
    test_json(&response);
}

#[test]
fn get_one_rating() {
    let e = Place::build().id("foo").finish();
    let (client, connections, mut search_engine, _) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_or_update_place(e)
        .unwrap();
    flows::create_rating(
        &connections,
        &mut *search_engine,
        usecases::NewPlaceRating {
            context: RatingContext::Humanity,
            value: RatingValue::from(2),
            user: None,
            title: "title".into(),
            entry: "foo".into(),
            comment: "bla".into(),
            source: Some("blabla".into()),
        },
    )
    .unwrap();
    let rid = connections
        .shared()
        .unwrap()
        .load_ratings_of_place("foo")
        .unwrap()[0]
        .id
        .clone();
    let req = client.get(format!("/ratings/{}", rid));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str.as_str().chars().next().unwrap(), '[');
    let ratings: Vec<json::Rating> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(ratings[0].comments.len(), 1);
    assert_eq!(ratings[0].id, rid.to_string());
}

#[test]
fn ratings_with_and_without_source() {
    let e1 = Place::build().id("foo").finish();
    let e2 = Place::build().id("bar").finish();
    let (client, connections, mut search_engine, _) = setup2();
    connections
        .exclusive()
        .unwrap()
        .create_or_update_place(e1)
        .unwrap();
    connections
        .exclusive()
        .unwrap()
        .create_or_update_place(e2)
        .unwrap();
    flows::create_rating(
        &connections,
        &mut *search_engine,
        usecases::NewPlaceRating {
            context: RatingContext::Humanity,
            value: RatingValue::from(2),
            user: None,
            title: "title".into(),
            entry: "foo".into(),
            comment: "bla".into(),
            source: Some("blabla blabla".into()),
        },
    )
    .unwrap();
    flows::create_rating(
        &connections,
        &mut *search_engine,
        usecases::NewPlaceRating {
            context: RatingContext::Humanity,
            value: RatingValue::from(2),
            user: None,
            title: "title".into(),
            entry: "bar".into(),
            comment: "bla".into(),
            source: Some("blabla blabla".into()),
        },
    )
    .unwrap();

    let rid = connections
        .shared()
        .unwrap()
        .load_ratings_of_place("bar")
        .unwrap()[0]
        .id
        .clone();
    let req = client.get(format!("/ratings/{}", rid));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    test_json(&response);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str.as_str().chars().next().unwrap(), '[');
    let ratings: Vec<json::Rating> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(ratings[0].id, rid.to_string());
    assert_eq!(ratings[0].comments.len(), 1);
}

pub fn cookie_from_response(response: &Response, key: &str) -> Option<Cookie<'static>> {
    let cookie = response
        .headers()
        .get("Set-Cookie")
        .find(|v| v.starts_with(key))
        .and_then(|val| Cookie::parse_encoded(val).ok());
    cookie.map(|c| c.into_owned())
}

// TODO: rename to account_cookie
fn user_id_cookie(response: &Response) -> Option<Cookie<'static>> {
    cookie_from_response(response, COOKIE_EMAIL_KEY)
}

#[test]
fn post_user() {
    let (client, _) = setup();
    let req1 = client
        .post("/users")
        .header(ContentType::JSON)
        .body(r#"{"email": "123412341234foo@bar.de", "password": "foo bar"}"#);
    let response1 = req1.dispatch();
    assert_eq!(response1.status(), Status::Ok);

    let req2 = client
        .post("/users")
        .header(ContentType::JSON)
        .body(r#"{"email": "123412341234baz@bar.de", "password": "baz bar"}"#);
    let response2 = req2.dispatch();
    assert_eq!(response2.status(), Status::Ok);
}

#[test]
fn login_with_invalid_credentials() {
    let (client, _) = setup();
    let req = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo", "password": "bar"}"#);
    let response = req.dispatch();
    assert!(!response
        .headers()
        .iter()
        .any(|h| h.name.as_str() == "Set-Cookie"));
    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
fn login_with_valid_credentials() {
    let (client, db) = setup();
    let users = vec![User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
        .dispatch();
    let cookie = user_id_cookie(&response).unwrap();
    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().len() > 25);
}

#[test]
fn login_logout_succeeds() {
    let (client, db) = setup();
    let users = vec![User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }

    // Login
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
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
#[cfg(feature = "jwt")]
fn login_logout_succeeds_jwt() {
    let (client, db) = setup();
    let users = vec![User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }

    // Login
    let res = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let jwt_token: ofdb_boundary::JwtToken = serde_json::from_str(&body_str).unwrap();

    // Logout
    let auth_header =
        rocket::http::Header::new("Authorization", format!("Bearer {}", jwt_token.token));
    let response = client
        .post("/logout")
        .header(ContentType::JSON)
        .header(auth_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Assert logout succeeded
    let res = client
        .get("/users/current")
        .header(ContentType::JSON)
        .header(auth_header)
        .dispatch();
    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
#[cfg(feature = "jwt")]
fn review_place_after_logout_must_fail() {
    let (client, db, mut search_engine, notify) = setup2();

    // Create user
    let user = User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Scout,
    };
    db.exclusive().unwrap().create_user(&user).unwrap();

    // Create place
    let new_place = usecases::NewPlace {
        title: "created".into(),
        ..default_new_entry()
    };
    let place_id = flows::create_place(
        &db,
        &mut *search_engine,
        &notify,
        new_place,
        None,
        None,
        &default_accepted_licenses(),
    )
    .unwrap()
    .id;

    // Login
    let res = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let body_str = res.into_string().unwrap();
    let jwt_token: ofdb_boundary::JwtToken = serde_json::from_str(&body_str).unwrap();

    // Rate place
    let req = client
        .post(format!("/places/{}/review", place_id))
        .header(ContentType::JSON)
        .body(format!(
            "{{\"status\":\"confirmed\",\"comment\":\"{place_id}\"}}"
        ));
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Logout
    let auth_header =
        rocket::http::Header::new("Authorization", format!("Bearer {}", jwt_token.token));
    let response = client
        .post("/logout")
        .header(ContentType::JSON)
        .header(auth_header.clone())
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Assert logout succeeded
    let res = client
        .get("/users/current")
        .header(ContentType::JSON)
        .header(auth_header.clone())
        .dispatch();
    assert_eq!(res.status(), Status::Unauthorized);

    let req = client
        .post(format!("/places/{}/review", place_id))
        .header(ContentType::JSON)
        .header(auth_header)
        .body(format!(
            "{{\"status\":\"rejected\",\"comment\":\"{place_id}\"}}"
        ));
    let res = req.dispatch();
    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
fn confirm_email_address() {
    let (client, db) = setup();
    let users = vec![User {
        email: "a@bar.de".parse().unwrap(),
        email_confirmed: false,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }

    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "a@bar.de", "password": "secret"}"#)
        .dispatch();

    assert_eq!(response.status(), Status::Forbidden);
    assert!(!db.exclusive().unwrap().all_users().unwrap()[0].email_confirmed);

    let token = EmailNonce {
        email: "a@bar.de".parse().unwrap(),
        nonce: Nonce::new(),
    }
    .encode_to_string();
    let response = client
        .post("/confirm-email-address")
        .header(ContentType::JSON)
        .body(format!("{{\"token\":\"{}\"}}", token))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(db.exclusive().unwrap().all_users().unwrap()[0].email_confirmed);

    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "a@bar.de", "password": "secret"}"#)
        .dispatch();
    let cookie = user_id_cookie(&response).expect("cookie");

    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().len() > 25);
}

//TODO: make it pass
#[ignore]
#[test]
fn send_confirmation_email() {
    let (client, db) = setup();
    let users = vec![User {
        email: "a@bar.de".parse().unwrap(),
        email_confirmed: false,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
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
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Guest,
    }];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
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
fn recently_changed_entries() {
    // Check that the requests succeeds on an empty database just
    // to verify that the literal SQL query that is not verified
    // at compile-time still matches the current database schema!
    let (client, db) = setup();

    let old_entries = vec![Place::build().id("old").finish()];
    for e in old_entries {
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }

    // Resolution of time stamps in the query is 1 sec
    // TODO: Don't waste time by sleeping
    std::thread::sleep(std::time::Duration::from_millis(1001));
    let since_inclusive = Timestamp::now();
    let recent_entries = vec![Place::build().id("recent").finish()];
    for e in recent_entries {
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }

    // Resolution of time stamps in the query is 1 sec
    // TODO: Don't waste time by sleeping
    std::thread::sleep(std::time::Duration::from_millis(1001));

    let until_exclusive = Timestamp::now();
    assert!(since_inclusive < until_exclusive);
    let new_entries = vec![Place::build().id("new").finish()];
    for e in new_entries {
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }

    let response_since = client
        .get(format!(
            "/entries/recently-changed?since={}",
            since_inclusive.as_secs(),
        ))
        .dispatch();
    assert_eq!(response_since.status(), Status::Ok);
    let body_since_str = response_since.into_string().unwrap();
    assert!(!body_since_str.contains("\"id\":\"old\""));
    assert!(body_since_str.contains("\"id\":\"recent\""));
    assert!(body_since_str.contains("\"id\":\"new\""));

    let response_until = client
        .get(format!(
            "/entries/recently-changed?until={}",
            until_exclusive.as_secs(),
        ))
        .dispatch();
    assert_eq!(response_until.status(), Status::Ok);
    let body_until_str = response_until.into_string().unwrap();
    assert!(body_until_str.contains("\"id\":\"old\""));
    assert!(body_until_str.contains("\"id\":\"recent\""));
    assert!(!body_until_str.contains("\"id\":\"new\""));

    let response_since_until = client
        .get(format!(
            "/entries/recently-changed?since={}&until={}",
            since_inclusive.as_secs(),
            until_exclusive.as_secs()
        ))
        .dispatch();
    assert_eq!(response_since_until.status(), Status::Ok);
    let body_since_until_str = response_since_until.into_string().unwrap();
    assert!(!body_since_until_str.contains("\"id\":\"old\""));
    assert!(body_since_until_str.contains("\"id\":\"recent\""));
    assert!(!body_since_until_str.contains("\"id\":\"new\""));
}

#[test]
fn count_most_popular_tags_on_empty_db_to_verify_sql() {
    // Check that the requests succeeds on an empty database just
    // to verify that the literal SQL query that is not verified
    // at compile-time still matches the current database schema!
    let (client, _) = setup();
    // All parameters
    let response = client
        .get("/entries/most-popular-tags?offset=10&limit=1000&min_count=10&max_count=100&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str, "[]");

    // Only offset parameter
    let response = client
        .get("/entries/most-popular-tags?offset=1&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Only limit parameter
    let response = client
        .get("/entries/most-popular-tags?limit=1&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Only min_count parameter
    let response = client
        .get("/entries/most-popular-tags?min_count=1&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Only max_count parameter
    let response = client
        .get("/entries/most-popular-tags?max_count=1&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
}

fn init_tags_cache_test_db(cnt: usize, db: &sqlite::Connections) {
    (1..=cnt)
        .map(|i| (1..=i).map(|i| i.to_string()).collect())
        .map(|tags| Place::build().tags(tags).finish())
        .for_each(|place| {
            db.exclusive()
                .unwrap()
                .create_or_update_place(place)
                .unwrap();
        });
}

#[test]
fn update_most_popular_tags_on_outdated_cache() {
    let (client, db) = setup();

    // init
    init_tags_cache_test_db(10, &db);

    // get only popular tags
    let response = client
        .get("/entries/most-popular-tags?min_count=8&max_cache_age=0")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    let tags: Vec<ofdb_boundary::TagFrequency> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(tags.len(), 3);

    let response = client
        .get("/entries/most-popular-tags?max_count=1")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    let tags: Vec<ofdb_boundary::TagFrequency> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(tags.len(), 1);

    // get cached popular tags again
    let response = client
        .get("/entries/most-popular-tags?min_count=8")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    let tags: Vec<ofdb_boundary::TagFrequency> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(tags.len(), 3);
}

#[test]
fn openapi() {
    let (client, _) = setup();
    let req = client.get("/server/openapi.yaml");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        response.headers().get("Content-Type").collect::<Vec<_>>()[0],
        "text/yaml"
    );
    let body_str = response.into_string().unwrap();
    assert!(body_str.contains("openapi:"))
}

#[test]
fn entries_export_csv() {
    let (client, db, mut search_engine, _) = setup2();

    let users = vec![
        User {
            email: "admin@example.com".parse().unwrap(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Admin,
        },
        User {
            email: "scout@example.com".parse().unwrap(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::Scout,
        },
        User {
            email: "user@example.com".parse().unwrap(),
            email_confirmed: true,
            password: "secret".parse::<Password>().unwrap(),
            role: Role::User,
        },
    ];
    for u in users {
        db.exclusive().unwrap().create_user(&u).unwrap();
    }

    let mut entries = vec![
        Place::build()
            .id("entry1")
            .license("license1")
            .title("title1")
            .description("desc1")
            .pos(MapPoint::from_lat_lng_deg(0.1, 0.2))
            .tags(vec![
                "bli",
                "bla",
                Category::TAG_NON_PROFIT,
                Category::TAG_COMMERCIAL,
            ])
            .finish(),
        Place::build()
            .id("entry2")
            .tags(vec![Category::TAG_NON_PROFIT])
            .finish(),
        Place::build()
            .id("entry3")
            .pos(MapPoint::from_lat_lng_deg(2.0, 2.0))
            .tags(vec![Category::TAG_COMMERCIAL])
            .finish(),
    ];
    entries[0].location.address = Some(Address::build().street("street1").finish());
    entries[0].created.at = Timestamp::try_from_secs(1111).unwrap();
    entries[0].created.by = Some("user@example.com".parse().unwrap());
    entries[0].contact = Some(Contact {
        name: Some("John Smith".to_string()),
        email: Some("john.smith@example.com".parse().unwrap()),
        phone: Some("0123456789".to_string()),
    });
    entries[0].location.address = Some(
        Address::build()
            .street("street1")
            .zip("zip1")
            .city("city1")
            .country("country1")
            .state("state1")
            .finish(),
    );
    entries[0].links = Some(Links {
        homepage: Some("http://homepage1".parse().unwrap()),
        image: Some("https://img".parse().unwrap()),
        image_href: Some("https://img,link".parse().unwrap()),
        custom: vec![CustomLink::from_url(
            "http://custom-link.org".parse().unwrap(),
        )],
    });
    entries[0].opening_hours = Some("24/7".parse().unwrap());
    entries[0].founded_on =
        Some(time::Date::from_calendar_date(1945, time::Month::October, 24).unwrap());
    entries[1].created.at = Timestamp::try_from_secs(2222).unwrap();

    db.exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "bli".into() })
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag { id: "bla".into() })
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: Category::TAG_NON_PROFIT.into(),
        })
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_tag_if_it_does_not_exist(&Tag {
            id: Category::TAG_COMMERCIAL.into(),
        })
        .unwrap();
    for e in entries {
        // Only works if all places have the default/initial revision!
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }
    let diversity = RatingContext::Diversity;
    db.exclusive()
        .unwrap()
        .create_rating(Rating {
            id: "123".into(),
            place_id: "entry1".into(),
            created_at: Timestamp::try_from_secs(123).unwrap(),
            archived_at: None,
            title: "rating1".into(),
            value: RatingValue::from(2),
            context: diversity,
            source: None,
        })
        .unwrap();
    db.exclusive()
        .unwrap()
        .create_rating(Rating {
            id: "345".into(),
            place_id: "entry1".into(),
            created_at: Timestamp::try_from_secs(123).unwrap(),
            archived_at: None,
            title: "rating2".into(),
            value: RatingValue::from(1),
            context: diversity,
            source: None,
        })
        .unwrap();

    let places = db.shared().unwrap().all_places().unwrap();
    for (place, status) in &places {
        let ratings = db
            .shared()
            .unwrap()
            .load_ratings_of_place(place.id.as_ref())
            .unwrap();
        search_engine
            .add_or_update_place(place, *status, &place.avg_ratings(&ratings))
            .unwrap();
    }
    search_engine.flush_index().unwrap();

    // Export as Admin (without token)
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "admin@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let req = client.get("/export/entries.csv?bbox=-1,-1,1,1");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers().iter() {
        if h.name.as_str() == "Content-Type" {
            assert_eq!(h.value, "text/csv; charset=utf-8");
        }
    }
    let body_str = response.into_string().unwrap();
    //eprintln!("{}", body_str);
    assert!(body_str.starts_with("id,created_at,created_by,version,title,description,lat,lng,street,zip,city,country,state,homepage,contact_name,contact_email,contact_phone,opening_hours,founded_on,categories,tags,license,image_url,image_link_url,avg_rating\n"));
    assert!(body_str.contains(&format!("entry1,1111,user@example.com,0,title1,desc1,{lat},{lng},street1,zip1,city1,country1,state1,http://homepage1/,John Smith,john.smith@example.com,0123456789,24/7,1945-10-24,\"{cat1},{cat2}\",\"bla,bli\",license1,https://img/,\"https://img,link/\",0.25\n", lat = LatCoord::from_deg(0.1).to_deg(), lng = LngCoord::from_deg(0.2).to_deg(), cat1 = Category::ID_NON_PROFIT, cat2 = Category::ID_COMMERCIAL)));
    assert!(body_str.contains(&format!(
        "entry2,2222,,0,,,0.0,0.0,,,,,,,,,,,,{cat},,,,,0.0\n",
        cat = Category::ID_NON_PROFIT
    )));
    assert!(!body_str.contains("entry3"));

    // Export as Scout (without token)
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "scout@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let req = client.get("/export/entries.csv?bbox=-1,-1,1,1");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers().iter() {
        if h.name.as_str() == "Content-Type" {
            assert_eq!(h.value, "text/csv; charset=utf-8");
        }
    }
    let body_str = response.into_string().unwrap();
    //eprintln!("{}", body_str);
    assert!(body_str.starts_with("id,created_at,created_by,version,title,description,lat,lng,street,zip,city,country,state,homepage,contact_name,contact_email,contact_phone,opening_hours,founded_on,categories,tags,license,image_url,image_link_url,avg_rating\n"));
    assert!(body_str.contains(&format!("entry1,1111,,0,title1,desc1,{lat},{lng},street1,zip1,city1,country1,state1,http://homepage1/,John Smith,john.smith@example.com,0123456789,24/7,1945-10-24,\"{cat1},{cat2}\",\"bla,bli\",license1,https://img/,\"https://img,link/\",0.25\n", lat = LatCoord::from_deg(0.1).to_deg(), lng = LngCoord::from_deg(0.2).to_deg(), cat1 = Category::ID_NON_PROFIT, cat2 = Category::ID_COMMERCIAL)));
    assert!(body_str.contains(&format!(
        "entry2,2222,,0,,,0.0,0.0,,,,,,,,,,,,{cat},,,,,0.0\n",
        cat = Category::ID_NON_PROFIT
    )));
    assert!(!body_str.contains("entry3"));

    // Export as User
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "user@example.com", "password": "secret"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    let req = client.get("/export/entries.csv?bbox=-1,-1,1,1");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn search_duplicates() {
    let (client, db) = setup();
    let res = client.post("/entries")
                    .header(ContentType::JSON)
                    .body(r#"{"title":"foo","description":"bla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":["test"]}"#)
                    .dispatch();
    assert_eq!(res.status(), Status::Ok);
    let (place, _) = db
        .shared()
        .unwrap()
        .all_places()
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let res = client
        .post("/search/duplicates")
        .header(ContentType::JSON)
        .body(r#"{"title":"foO","description":"bla","lat":0.0005,"lng":0.0005}"#)
        .dispatch();
    assert_eq!(res.status(), Status::Ok);
    test_json(&res);
    let body_str = res.into_string().unwrap();
    let duplicate_places: Vec<ofdb_boundary::PlaceSearchResult> =
        serde_json::from_str(&body_str).unwrap();
    assert_eq!(1, duplicate_places.len());
    assert_eq!(place.id.to_string(), duplicate_places.first().unwrap().id);
}

#[test]
fn get_version() {
    let (client, _) = setup();
    let req = client.get("/server/version");
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.into_string().unwrap();
    assert_eq!(body_str, DUMMY_VERSION);
}

mod with_captcha_protection_enabled {
    use super::*;

    fn captcha_setup() -> (Client, sqlite::Connections) {
        let cfg = Cfg {
            protect_with_captcha: true,
            accepted_licenses: default_accepted_licenses(),
        };
        setup_with_cfg(cfg)
    }

    #[test]
    fn create_place_without_captcha_cookie() {
        let (client, _) = captcha_setup();
        let req = client.post("/entries")
                        .header(ContentType::JSON)
                        .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":[]}"#);
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn create_place_with_valid_captcha_cookie() {
        let (client, db) = captcha_setup();
        let cookie = get_captcha_cookie(&client).unwrap();
        let req = client.post("/entries")
                        .header(ContentType::JSON)
                        .cookie(cookie)
                        .body(r#"{"title":"foo","description":"blablabla","lat":0.0,"lng":0.0,"categories":["x"],"license":"CC0-1.0","tags":[]}"#);
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
        test_json(&response);
        let body_str = response.into_string().unwrap();
        let eid = db.exclusive().unwrap().all_places().unwrap()[0]
            .0
            .id
            .clone();
        assert_eq!(body_str, format!("\"{}\"", eid));
    }
}

#[test]
fn not_updated_since() {
    let (client, db) = setup();

    // Create admin user and login
    let user = User {
        email: "foo@bar".parse().unwrap(),
        email_confirmed: true,
        password: "secret".parse::<Password>().unwrap(),
        role: Role::Admin,
    };
    db.exclusive().unwrap().create_user(&user).unwrap();
    let response = client
        .post("/login")
        .header(ContentType::JSON)
        .body(r#"{"email": "foo@bar", "password": "secret"}"#)
        .dispatch();
    assert_eq!(response.status(), Status::Ok);

    // Create some places with reviews
    let old_entries = vec![
        Place::build().id("old").finish(),
        Place::build().id("archived").finish(),
        Place::build().id("rejected").finish(),
    ];
    for e in old_entries {
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }
    for (id, status) in &[("archived", "archived"), ("rejected", "rejected")] {
        let req = client
            .post(format!("/places/{}/review", id))
            .header(ContentType::JSON)
            .body(format!(
                "{{\"status\":\"{}\",\"comment\":\"{}\"}}",
                status, id
            ));
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    // Resolution of time stamps in the query is 1 sec
    // TODO: Don't waste time by sleeping
    std::thread::sleep(std::time::Duration::from_millis(1001));
    let update_time = Timestamp::now();
    let recent_entries = vec![Place::build().id("recent").finish()];
    for e in recent_entries {
        db.exclusive().unwrap().create_or_update_place(e).unwrap();
    }

    let response_since = client
        .get(format!(
            "/places/not-updated?since={}",
            update_time.as_secs()
        ))
        .dispatch();
    assert_eq!(response_since.status(), Status::Ok);
    let body_since_str = response_since.into_string().unwrap();
    assert!(!body_since_str.contains("\"id\":\"recent\""));
    assert!(body_since_str.contains("\"id\":\"old\""));
    assert!(!body_since_str.contains("\"id\":\"archived\""));
    assert!(!body_since_str.contains("\"id\":\"rejected\""));
}

#[test]
fn review_place_with_token() {
    let (client, db) = setup();
    let place_id = create_place(&client);
    let place_revision = Revision::initial();
    let nonce = Nonce::new();
    let expires_at = Timestamp::now() + time::Duration::seconds(10);
    let review_nonce = ReviewNonce {
        place_id,
        place_revision,
        nonce,
    };
    let review_token = ReviewToken {
        expires_at,
        review_nonce,
    };
    db.exclusive()
        .unwrap()
        .add_review_token(&review_token)
        .unwrap();
    let token = review_token.review_nonce.encode_to_string();
    let res = client
        .post("/places/review-with-token")
        .header(ContentType::JSON)
        .body(format!("{{\"token\":\"{token}\",\"status\":\"archived\"}}",))
        .dispatch();
    // TODO: should be Status::Created
    test_json(&res);
    assert_eq!(res.status(), Status::Ok);
}

#[test]
fn review_place_with_token_and_invalid_status() {
    let (client, db) = setup();
    let place_id = create_place(&client);
    let place_revision = Revision::initial();
    let nonce = Nonce::new();
    let expires_at = Timestamp::now() + time::Duration::seconds(10);
    let review_nonce = ReviewNonce {
        place_id,
        place_revision,
        nonce,
    };
    let review_token = ReviewToken {
        expires_at,
        review_nonce,
    };
    db.exclusive()
        .unwrap()
        .add_review_token(&review_token)
        .unwrap();
    let token = review_token.review_nonce.encode_to_string();
    let res = client
        .post("/places/review-with-token")
        .header(ContentType::JSON)
        .body(format!(
            "{{\"token\":\"{token}\",\"status\":\"doesnotexist\"}}",
        ))
        .dispatch();
    test_json(&res);
    assert_eq!(res.status(), Status::UnprocessableEntity);
}

#[test]
fn review_place_with_token_and_invalid_revision() {
    let (client, db) = setup();
    let place_id = create_place(&client);
    let place_revision = Revision::from(500);
    let nonce = Nonce::new();
    let expires_at = Timestamp::now() + time::Duration::seconds(10);
    let review_nonce = ReviewNonce {
        place_id,
        place_revision,
        nonce,
    };
    let review_token = ReviewToken {
        expires_at,
        review_nonce,
    };
    db.exclusive()
        .unwrap()
        .add_review_token(&review_token)
        .unwrap();
    let token = review_token.review_nonce.encode_to_string();
    let res = client
        .post("/places/review-with-token")
        .header(ContentType::JSON)
        .body(format!(
            "{{\"token\":\"{token}\",\"status\":\"confirmed\"}}",
        ))
        .dispatch();
    test_json(&res);
    assert_eq!(res.status(), Status::BadRequest);
}
