use rocket::LoggingLevel;
use rocket::config::{Environment, Config};
use rocket::local::Client;
use rocket::http::{Status, ContentType};
use business::db::Db;
use business::builder::*;
use infrastructure;
use serde_json;
use super::*;
use pwhash::bcrypt;

fn setup() -> (Client, mockdb::ConnectionPool) {
    let cfg = Config::build(Environment::Development)
        .log_level(LoggingLevel::Debug)
        .finalize()
        .unwrap();
    let pool = mockdb::create_connection_pool().unwrap();
    let rocket = super::rocket_instance(cfg, pool.clone());
    let client = Client::new(rocket).unwrap();
    (client, pool)
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
    let (client, db) = setup();
    db.get().unwrap().create_entry(&e).unwrap();
    usecase::rate_entry(&mut *db.get().unwrap(), usecase::RateEntry{
        context : RatingContext::Humanity,
        value   : 2,
        title   : "title".into(),
        user    : None,
        entry   : "get_one_entry_test".into(),
        comment : "bla".into(),
    }).unwrap();
    let req = client.get("/entries/get_one_entry_test");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().iter().any(|h|h.name.as_str() == "Content-Type"));
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let entries: Vec<Entry> = serde_json::from_str(&body_str).unwrap();
    let rid = db.get().unwrap().ratings[0].id.clone();
    assert!(body_str.contains(&format!(r#""ratings":["{}"]"#,rid)));
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
    let (client, db) = setup();
    db.get().unwrap().create_entry(&one).unwrap();
    db.get().unwrap().create_entry(&two).unwrap();
    let req = client.get("/entries/get_multiple_entry_test_one,get_multiple_entry_test_two");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().iter().any(|h|h.name.as_str() == "Content-Type"));
    for h in response.headers().iter() {
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
    let (client, db) = setup();
    db.get().unwrap().entries = entries;
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
fn search_with_tags() {
    let entries = vec![
        Entry::build().id("a").categories(vec!["foo"]).finish(),
        Entry::build().id("b").categories(vec!["foo"]).finish(),    // bla-blubb, foo-bar
        Entry::build().id("c").categories(vec!["foo"]).finish(),    // foo-bar
    ];
    let triples = vec![
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bla-blubb".into())},
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())},
        Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())}
    ];
    let (client, db) = setup();
    db.get().unwrap().entries = entries;
    db.get().unwrap().triples = triples;
    let req = client.get("/search?bbox=-10,-10,10,10&tags=bla-blubb");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":["b"]"#));

    let req = client.get("/search?bbox=-10,-10,10,10&tags=foo-bar");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains("\"b\""));
    assert!(body_str.contains("\"c\""));
}

#[test]
fn search_with_hash_tags() {
    let entries = vec![
        Entry::build().id("a").finish(),
        Entry::build().id("b").finish(),    // bla-blubb, foo-bar
        Entry::build().id("c").finish(),    // foo-bar
    ];
    let triples = vec![
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bla-blubb".into())},
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())},
        Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())}
    ];
    let (client, db) = setup();
    db.get().unwrap().entries = entries;
    db.get().unwrap().triples = triples;
    let req = client.get("/search?bbox=-10,-10,10,10&text=%23bla-blubb");
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert!(body_str.contains(r#""visible":["b"]"#));
}

#[test]
fn search_with_and_without_tags() {
    let entries = vec![
        Entry::build().id("a").title("foo").finish(),
        Entry::build().id("b").title("foo").finish(),    // bla-blubb, foo-bar
        Entry::build().id("c").title("foo").finish(),    // foo-bar
    ];
    let triples = vec![
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("bla-blubb".into())},
        Triple{ subject: ObjectId::Entry("b".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())},
        Triple{ subject: ObjectId::Entry("c".into()), predicate: Relation::IsTaggedWith, object: ObjectId::Tag("foo-bar".into())}
    ];
    let (client, db) = setup();
    db.get().unwrap().entries = entries;
    db.get().unwrap().triples = triples;
    let req = client.get("/search?bbox=-10,-10,10,10&text=bla-blubb");
    let mut response = req.dispatch();
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
    let (client, db) = setup();
    let req = client.post("/users")
        .header(ContentType::JSON)
        .body(r#"{"username":"foo","email":"foo@bar.com","password":"bar"}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    let u = db.get().unwrap().get_user("foo").unwrap();
    assert_eq!(u.username, "foo");
    assert!(bcrypt::verify("bar", &u.password));
}

#[test]
fn create_rating() {
    let (client, db) = setup();
    db.get().unwrap().entries = vec![ Entry::build().id("foo").finish() ];
    let req = client.post("/ratings")
        .header(ContentType::JSON)
        .body(r#"{"value": 1,"context":"fairness","entry":"foo","comment":"test", "title":"idontcare"}"#);
    let response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(db.get().unwrap().ratings[0].value, 1);
}

#[test]
fn get_one_rating() {
    let e = Entry::build().id("foo").finish();
    let (client, db) = setup();
    db.get().unwrap().create_entry(&e).unwrap();
    usecase::rate_entry(&mut *db.get().unwrap(), usecase::RateEntry{
        context : RatingContext::Humanity,
        value   : 2,
        user    : None,
        title   : "title".into(),
        entry   : "foo".into(),
        comment : "bla".into(),
    }).unwrap();
    let rid = db.get().unwrap().ratings[0].id.clone();
    let req = client.get(format!("/ratings/{}",rid));
    let mut response = req.dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert!(response.headers().iter().any(|h|h.name.as_str() == "Content-Type"));
    for h in response.headers().iter() {
        match h.name.as_str() {
            "Content-Type" => assert_eq!(h.value, "application/json"),
            _ => { /* let these through */ }
        }
    }
    let body_str = response.body().and_then(|b| b.into_string()).unwrap();
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '[');
    let ratings: Vec<json::Rating> = serde_json::from_str(&body_str).unwrap();
    assert_eq!(ratings[0].id,rid);
    assert_eq!(ratings[0].comments.len(),1);
}

#[test]
fn login_with_invalid_credentials() {
    let (client, _) = setup();
    let req = client.post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#);
    let response = req.dispatch();
    assert!(!response.headers().iter().any(|h|h.name.as_str() == "Set-Cookie"));
    assert_eq!(response.status(), Status::Unauthorized);
}

#[test]
fn login_with_valid_credentials() {
    let (client, db) = setup();
    db.get().unwrap().users = vec![
        User{
            username: "foo".into(),
            password: bcrypt::hash("bar").unwrap(),
            email: "foo@bar".into()
        }];
    let response = client.post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let cookie : Cookie = response
                            .headers()
                            .iter()
                            .filter(|h|h.name == "Set-Cookie")
                            .filter(|h|h.value.contains("user_id="))
                            .nth(0)
                            .unwrap()
                            .value
                            .parse()
                            .unwrap();

    assert_eq!(response.status(), Status::Ok);
    assert!(cookie.value().len() > 25);
}

// TODO: make this test pass!
#[ignore]
#[test]
fn logout() {
    let (client, db) = setup();
    db.get().unwrap().users = vec![
        User{ username: "foo".into(), password: bcrypt::hash("bar").unwrap(), email: "foo@bar".into() }
    ];
    let response = client.post("/login")
        .header(ContentType::JSON)
        .body(r#"{"username": "foo", "password": "bar"}"#)
        .dispatch();
    let user_id : String = response
                            .headers()
                            .iter()
                            .filter(|h|h.name == "Set-Cookie")
                            .filter(|h|h.value.contains("user_id="))
                            .nth(0)
                            .unwrap()
                            .value
                            .parse::<Cookie>()
                            .unwrap()
                            .value()
                            .into();
    let response = client
                        .post("/logout")
                        .header(ContentType::JSON)
                        .cookie(Cookie::new("user_id", user_id))
                        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response
        .headers()
        .iter()
        .filter(|h|h.name == "Set-Cookie")
        .filter(|h|h.value.contains("user_id="))
        .nth(0)
        .unwrap()
        .value
        .parse::<Cookie>()
        .unwrap()
        .value(), "");
}

#[test]
fn to_words(){
    let text = "blabla bla-blubb #foo-bar";
    let words = infrastructure::web::to_words(&text);
    assert_eq!(words.len(), 3);
    assert_eq!(words[0], "blabla");
    assert_eq!(words[1], "bla-blubb");
    assert_eq!(words[2], "#foo-bar");
}
