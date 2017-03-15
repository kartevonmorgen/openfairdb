use super::rocket;
use rocket::testing::MockRequest;
use rocket::http::{Status, Method};
use entities::*;
use business::db::{Db,Repo};
use serde_json;

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
    super::db().unwrap().conn().create_entry(&e).unwrap();
    let rocket = rocket::ignite().mount("/", routes![super::get_entries]);
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
    assert!(body_str.contains("\"id\":\"get_all_entries_test\""));
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
    super::db().unwrap().conn().create_entry(&e).unwrap();
    let rocket = rocket::ignite().mount("/", routes![super::get_entry]);
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
    assert_eq!(body_str.as_str().chars().nth(0).unwrap(), '{');
    let entry: Entry = serde_json::from_str(&body_str).unwrap();
    assert!(entry==e);
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
    super::db().unwrap().conn().create_entry(&one).unwrap();
    super::db().unwrap().conn().create_entry(&two).unwrap();
    let rocket = rocket::ignite().mount("/", routes![super::get_entry]);
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
