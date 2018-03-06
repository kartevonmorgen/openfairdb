use rocket::logger::LoggingLevel;
use rocket::config::{Config, Environment};
use rocket::local::Client;
use rocket::http::{ContentType, Cookie, Status};
use business::db::Db;
use business::builder::*;
use business::usecase;
use serde_json;
use entities::*;
use adapters::json;
use rocket::response::Response;
use super::util::*;
use pwhash::bcrypt;
use test::Bencher;
use super::sqlite;
use uuid::Uuid;
use std::fs;

fn setup() -> (Client, sqlite::ConnectionPool) {
    let cfg = Config::build(Environment::Development)
        .log_level(LoggingLevel::Debug)
        .finalize()
        .unwrap();
    let uuid = Uuid::new_v4().simple().to_string();
    fs::create_dir_all("test-dbs").unwrap();
    let pool = sqlite::create_connection_pool(&format!("./test-dbs/{}", uuid)).unwrap();
    let rocket = super::rocket_instance(cfg, pool.clone());
    let client = Client::new(rocket).unwrap();
    (client, pool)
}

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
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    let eid = db.get().unwrap().all_entries().unwrap()[0].id.clone();
    assert_eq!(body_str, format!("\"{}\"", eid));
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
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
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
    db.get().unwrap().create_entry(&e).unwrap();
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
    ).unwrap();
    let req = client.get("/entries/get_one_entry_test");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    let rid = db.get().unwrap().all_ratings().unwrap()[0].id.clone();
    assert!(body_str.contains(&format!(r#""ratings":["{}"]"#, rid)));
    assert!(entries[0] == e);
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
    db.get().unwrap().create_entry(&one).unwrap();
    db.get().unwrap().create_entry(&two).unwrap();
    let req = client.get("/entries/get_multiple_entry_test_one,get_multiple_entry_test_two");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|x| *x == one));
    assert!(entries.iter().any(|x| *x == two));
}

#[test]
fn search_with_categories() {
    let entries = vec![
        Entry::build().id("a").categories(vec!["foo"]).finish(),
        Entry::build().id("b").categories(vec!["foo"]).finish(),
        Entry::build().id("c").categories(vec!["bar"]).finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_category_if_it_does_not_exist(&Category {
        id: "foo".into(),
        created: 0,
        version: 0,
        name: "foo".into(),
    }).unwrap();
    conn.create_category_if_it_does_not_exist(&Category {
        id: "bar".into(),
        created: 0,
        version: 0,
        name: "bar".into(),
    }).unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&categories=foo");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"a\""));
    assert!(!body_str.contains("\"c\""));

    let req = client.get("/search?bbox=-10,-10,10,10&categories=bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains("\"b\""));
    assert!(!body_str.contains("\"a\""));
    assert!(body_str.contains("\"c\""));

    let req = client.get("/search?bbox=-10,-10,10,10");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"a\""));
    assert!(body_str.contains("\"c\""));
}

#[test]
fn search_with_text() {
    let entries = vec![
        Entry::build()
            .title("Foo")
            .description("bla")
            .id("a")
            .finish(),
        Entry::build()
            .title("bar")
            .description("foo")
            .id("b")
            .finish(),
        Entry::build()
            .title("baZ")
            .description("blub")
            .id("c")
            .finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&text=Foo");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"a\""));
    assert!(body_str.contains("\"b\""));
    assert!(!body_str.contains("\"c\""));
}

#[ignore]
#[bench]
fn bench_search_in_10_000_rated_entries(b: &mut Bencher) {
    let (entries, ratings) = ::business::sort::tests::create_entries_with_ratings(10_000);
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    for r in ratings {
        conn.create_rating(&r).unwrap();
    }
    b.iter(|| client.get("/search?bbox=-10,-10,10,10").dispatch());
}

#[test]
fn search_with_tags() {
    let entries = vec![
        Entry::build().id("a").categories(vec!["foo"]).finish(),
        Entry::build()
            .id("b")
            .tags(vec!["bla-blubb", "foo-bar"])
            .categories(vec!["foo"])
            .finish(),
        Entry::build()
            .id("c")
            .tags(vec!["foo-bar"])
            .categories(vec!["foo"])
            .finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_category_if_it_does_not_exist(&Category {
        id: "foo".into(),
        created: 0,
        version: 0,
        name: "foo".into(),
    }).unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    }).unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    }).unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&tags=bla-blubb");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":[{"id":"b","lat":0.0,"lng":0.0}]"#,));

    let req = client.get("/search?bbox=-10,-10,10,10&tags=foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#"{"id":"b","#));
    assert!(body_str.contains(r#"{"id":"c","#));
}

#[test]
fn search_with_uppercase_tags() {
    let entries = vec![
        Entry::build().tags(vec!["foo"]).id("a").finish(),
        Entry::build().tags(vec!["bar"]).id("b").finish(),
        Entry::build().tags(vec!["baz"]).id("c").finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag { id: "foo".into() })
        .unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag { id: "bar".into() })
        .unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag { id: "baz".into() })
        .unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&tags=Foo");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"a\""));
    assert!(!body_str.contains("\"b\""));
    assert!(!body_str.contains("\"c\""));
}

#[test]
fn search_with_hashtag() {
    let entries = vec![
        Entry::build().id("a").finish(),
        Entry::build()
            .id("b")
            .tags(vec!["bla-blubb", "foo-bar"])
            .finish(),
        Entry::build().id("c").tags(vec!["foo-bar"]).finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    }).unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    }).unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(
        body_str.contains(
            r#""visible":[{"id":"b","lat":0.0,"lng":0.0},{"id":"c","lat":0.0,"lng":0.0}]"#,
        )
    );
}

#[test]
fn search_with_two_hashtags() {
    let entries = vec![
        Entry::build().id("a").finish(),
        Entry::build()
            .tags(vec!["bla-blubb", "foo-bar"])
            .id("b")
            .finish(),
        Entry::build().tags(vec!["foo-bar"]).id("c").finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    }).unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    }).unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23bla-blubb %23foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":[{"id":"b","lat":0.0,"lng":0.0}]"#,));
}

#[test]
// TODO
#[ignore]
fn search_without_specifying_hashtag_symbol() {
    let entries = vec![
        Entry::build().id("a").title("foo").finish(),
        Entry::build()
            .id("b")
            .tags(vec!["bla-blubb", "foo-bar"])
            .title("foo")
            .finish(),
        Entry::build()
            .id("c")
            .tags(vec!["foo-bar"])
            .title("foo")
            .finish(),
    ];
    let (client, db) = setup();
    let mut conn = db.get().unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "bla-blubb".into(),
    }).unwrap();
    conn.create_tag_if_it_does_not_exist(&Tag {
        id: "foo-bar".into(),
    }).unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let mut response = client
        .get("/search?bbox=-10,-10,10,10&text=bla-blubb")
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":[{"id":"b","lat":0.0,"lng":0.0}]"#,));
}

#[test]
fn extract_ids_test() {
    assert_eq!(extract_ids("abc"), vec!["abc"]);
    assert_eq!(extract_ids("a,b,c"), vec!["a", "b", "c"]);
    assert_eq!(extract_ids("").len(), 0);
    assert_eq!(extract_ids("abc,,d"), vec!["abc", "d"]);
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
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
}

#[test]
fn create_rating() {
    let (client, db) = setup();
    let entries = vec![Entry::build().id("foo").finish()];
    let mut conn = db.get().unwrap();
    for e in entries {
        conn.create_entry(&e).unwrap();
    }
    let req = client.post("/ratings")
        .header(ContentType::JSON)
        .body(r#"{"value": 1,"context":"fairness","entry":"foo","comment":"test", "title":"idontcare", "source":"source..."}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(db.get().unwrap().all_ratings().unwrap()[0].value, 1);
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
}

#[test]
fn get_one_rating() {
    let e = Entry::build().id("foo").finish();
    let (client, db) = setup();
    db.get().unwrap().create_entry(&e).unwrap();
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
    ).unwrap();
    let rid = db.get().unwrap().all_ratings().unwrap()[0].id.clone();
    let req = client.get(format!("/ratings/{}", rid));
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
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
    db.get().unwrap().create_entry(&e1).unwrap();
    db.get().unwrap().create_entry(&e2).unwrap();
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
    ).unwrap();
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
    ).unwrap();

    let rid = db.get().unwrap().all_ratings().unwrap()[0].id.clone();
    let req = client.get(format!("/ratings/{}", rid));
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
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
    let users = vec![
        User {
            id: "123".into(),
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "foo@bar".into(),
            email_confirmed: true,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
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
    let users = vec![
        User {
            id: "123".into(),
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "foo@bar".into(),
            email_confirmed: true,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
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
        },
        User {
            id: "123".into(),
            username: "b".into(),
            password: bcrypt::hash("b").unwrap(),
            email: "b@bar".into(),
            email_confirmed: true,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
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
    assert!(
        response
            .headers()
            .iter()
            .any(|h| h.name.as_str() == "Content-Type")
    );
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
}

#[test]
fn confirm_email_address() {
    let (client, db) = setup();
    let users = vec![
        User {
            id: "123".into(),
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "a@bar.de".into(),
            email_confirmed: false,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
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
    let users = vec![
        User {
            id: "123".into(),
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "a@bar.de".into(),
            email_confirmed: false,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
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
    let users = vec![
        User {
            id: "123".into(),
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "foo@bar".into(),
            email_confirmed: true,
        },
    ];
    let mut conn = db.get().unwrap();
    for u in users {
        conn.create_user(&u).unwrap();
        conn.confirm_email_address("123").unwrap();
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
