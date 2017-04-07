use rocket::{Rocket, LoggingLevel};
use rocket::config::{Environment, Config};
use rocket::testing::MockRequest;
use rocket::http::{Status, Method, ContentType};
use business::db::Db;
use business::builder::*;
use serde_json;
use super::*;
use pwhash::bcrypt;

fn server() -> (Rocket, mockdb::ConnectionPool) {
    let cfg = Config::build(Environment::Development)
        .log_level(LoggingLevel::Debug)
        .finalize()
        .unwrap();
    let pool = mockdb::create_connection_pool().unwrap();
    let rocket = super::rocket_instance(cfg, pool.clone());
    (rocket, pool)
}

#[test]
fn get_all_entries() {
    let e = Entry{
        id          :  "get_all_entries_test".into(),
        created     :  0,
        version     :  0,
        title       :  "some".into(),
        description :  "desc".into(),
        lat         :  0.0,
        lng         :  0.0,
        street      :  None,
        zip         :  None,
        city        :  None,
        country     :  None,
        email       :  None,
        telephone   :  None,
        homepage    :  None,
        categories  :  vec![],
        license     :  None,
    };
    let (rocket, pool) = server();
    pool.get().unwrap().create_entry(&e).unwrap();
    let mut req = MockRequest::new(Method::Get, "/entries");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    assert!(body_str.contains(r#""id":"get_all_entries_test""#));
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    assert!(entries.iter().any(|x|*x==e));
}

#[test]
fn get_one_entry() {
    let e = Entry{
        id          :  "get_one_entry_test".into(),
        created     :  0,
        version     :  0,
        title       :  "some".into(),
        description :  "desc".into(),
        lat         :  0.0,
        lng         :  0.0,
        street      :  None,
        zip         :  None,
        city        :  None,
        country     :  None,
        email       :  None,
        telephone   :  None,
        homepage    :  None,
        categories  :  vec![],
        license     :  None,
    };
    let (rocket, db) = server();
    db.get().unwrap().create_entry(&e).unwrap();
    let mut req = MockRequest::new(Method::Get, "/entries/get_one_entry_test");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    assert!(entries[0]==e);
}

#[test]
fn get_multiple_entries() {
    let one = Entry{
        id          :  "get_multiple_entry_test_one".into(),
        created     :  0,
        version     :  0,
        title       :  "some".into(),
        description :  "desc".into(),
        lat         :  0.0,
        lng         :  0.0,
        street      :  None,
        zip         :  None,
        city        :  None,
        country     :  None,
        email       :  None,
        telephone   :  None,
        homepage    :  None,
        categories  :  vec![],
        license     :  None,
    };
    let two = Entry{
        id          :  "get_multiple_entry_test_two".into(),
        created     :  0,
        version     :  0,
        title       :  "some".into(),
        description :  "desc".into(),
        lat         :  0.0,
        lng         :  0.0,
        street      :  None,
        zip         :  None,
        city        :  None,
        country     :  None,
        email       :  None,
        telephone   :  None,
        homepage    :  None,
        categories  :  vec![],
        license     :  None,
    };
    let (rocket, db) = server();
    db.get().unwrap().create_entry(&one).unwrap();
    db.get().unwrap().create_entry(&two).unwrap();
    let mut req = MockRequest::new(Method::Get, "/entries/get_multiple_entry_test_one,get_multiple_entry_test_two");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    for h in response.headers() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(entries.len(),2);
    assert!(entries.iter().any(|x|*x==one));
    assert!(entries.iter().any(|x|*x==two));
}

#[test]
fn search_with_categories() {
    let entries = vec![
        Entry::build().id("a").categories(vec!["foo"]).finish(),
        Entry::build().id("b").categories(vec!["foo"]).finish(),
        Entry::build().id("c").categories(vec!["bar"]).finish(),
    ];
    let (rocket, db) = server();
    db.get().unwrap().entries = entries;
    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10&categories=foo");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"a\""));
    assert!(!body_str.contains("\"c\""));

    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10&categories=bar");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(!body_str.contains("\"b\""));
    assert!(!body_str.contains("\"a\""));
    assert!(body_str.contains("\"c\""));

    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"a\""));
    assert!(body_str.contains("\"c\""));
}

#[test]
fn search_with_tags() {
    let entries = vec![
        Entry::build().id("a").categories(vec!["foo"]).finish(),
        Entry::build().id("b").categories(vec!["foo"]).finish(),
        Entry::build().id("c").categories(vec!["foo"]).finish(),
    ];
    let triples = vec![
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bio".into())},
        Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bio".into())}
    ];
    let (rocket, db) = server();
    db.get().unwrap().entries = entries;
    db.get().unwrap().triples = triples;
    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10&tags=csa");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":["b"]"#));

    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10&tags=bio");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"c\""));
    assert!(!body_str.contains("\"a\""));
}

#[test]
fn search_with_hash_tags() {
    let entries = vec![
        Entry::build().id("a").finish(),
        Entry::build().id("b").finish(),
        Entry::build().id("c").finish(),
    ];
    let triples = vec![
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("csa".into())},
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bio".into())},
        Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bio".into())}
    ];
    let (rocket, db) = server();
    db.get().unwrap().entries = entries;
    db.get().unwrap().triples = triples;
    let mut req = MockRequest::new(Method::Get, "/search?bbox=-10,-10,10,10&text=%23csa");
    let mut response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":["b"]"#));
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
    assert_eq!(extract_hash_tags("none").len(),0);
    assert_eq!(extract_hash_tags("#").len(),0);
    assert_eq!(extract_hash_tags("foo #bar none"),vec!["bar".to_string()]);
    assert_eq!(extract_hash_tags("foo #bar,none"),vec!["bar".to_string()]);
    assert_eq!(extract_hash_tags("foo#bar,none"),vec!["bar".to_string()]);
    assert_eq!(extract_hash_tags("foo#bar none#baz"),vec!["bar".to_string(),"baz".to_string()]);
    assert_eq!(extract_hash_tags("#bar#baz"),vec!["bar".to_string(),"baz".to_string()]);
    assert_eq!(extract_hash_tags("#a-long-tag#baz"),vec!["a-long-tag".to_string(),"baz".to_string()]);
    assert_eq!(extract_hash_tags("#-").len(),0);
    assert_eq!(extract_hash_tags("#tag-"),vec!["tag".to_string()]);
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
    let (rocket, db) = server();
    let mut req = MockRequest::new(Method::Post, "/users")
        .header(ContentType::JSON)
        .body(r#"{"username":"foo","email":"foo@bar.com","password":"bar"}"#);
    let response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    let u = db.get().unwrap().get_user("foo").unwrap();
    assert_eq!(u.username, "foo");
    assert!(bcrypt::verify("bar", &u.password));
}

#[test]
fn create_rating() {
    let (rocket, db) = server();
    db.get().unwrap().entries = vec![ Entry::build().id("foo").finish() ];
    let mut req = MockRequest::new(Method::Post, "/ratings")
        .header(ContentType::JSON)
        .body(r#"{"value": 1,"context":"fair","entry":"foo","comment":"test"}"#);
    let response = req.dispatch_with(&rocket);
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(db.get().unwrap().ratings[0].value,1);
}
