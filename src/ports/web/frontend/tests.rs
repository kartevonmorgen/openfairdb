use super::*;
use crate::{
    infrastructure::db::{sqlite::Connections, tantivy},
    ports::web::tests::{prelude::*, register_user},
};

fn setup() -> (
    rocket::local::Client,
    sqlite::Connections,
    tantivy::SearchEngine,
) {
    crate::ports::web::tests::setup(vec![("/", super::routes())])
}

fn create_user(pool: &Connections, name: &str, role: Role) {
    let email = format!("{}@example.com", name);
    register_user(&pool, &email, "secret", true);
    let mut user = get_user(pool, name);
    user.role = role;
    pool.exclusive().unwrap().update_user(&user).unwrap();
}

fn get_user(pool: &Connections, name: &str) -> User {
    let email = format!("{}@example.com", name);
    pool.shared()
        .unwrap()
        .try_get_user_by_email(&email)
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
}

fn login_user(client: &Client, name: &str) {
    client
        .post("/login")
        .header(ContentType::Form)
        .body(format!("email={}%40example.com&password=secret", name))
        .dispatch();
}

mod events {
    use super::*;
    use chrono::prelude::*;

    use crate::infrastructure::flows::prelude as flows;

    #[test]
    fn search_events() {
        let (client, db, mut search_engine) = setup();
        let new_events = vec![
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::days(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["foo".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::hours(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["bla".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
            usecases::NewEvent {
                title: "foo".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(1))
                        .unwrap(),
                )
                .into_inner(),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["foo".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
        ];
        let event_ids = {
            let mut event_ids = Vec::with_capacity(new_events.len());
            for e in new_events {
                let e = flows::create_event(&db, &mut search_engine, None, e).unwrap();
                event_ids.push(e.id);
            }
            event_ids
        };

        // All events
        let mut res = client.get("/events").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with simple text
        let mut res = client.get("/events?text=foo").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with hashtag text
        let mut res = client.get("/events?text=%23foo").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with tag
        let mut res = client.get("/events?tag=foo").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with simple text (not found)
        let mut res = client.get("/events?text=bar").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with hashtag text (not found)
        let mut res = client.get("/events?text=%23bar").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));

        // Search with tag (not found)
        let mut res = client.get("/events?tag=bar").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[3])));
    }

    #[test]
    fn get_a_list_of_events_filtered_by_tags() {
        let (client, db, mut search_engine) = setup();
        let new_events = vec![
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::hours(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["bla".into(), "blub".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::days(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["bli".into(), "blub".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
            usecases::NewEvent {
                title: "x".into(),
                start: Timestamp::from(
                    chrono::Utc::now()
                        .checked_sub_signed(chrono::Duration::days(2))
                        .unwrap(),
                )
                .into_inner(),
                tags: Some(vec!["blub".into()]),
                registration: Some("email".into()),
                email: Some("test@example.com".into()),
                created_by: Some("test@example.com".into()),
                ..Default::default()
            },
        ];
        let event_ids = {
            let mut event_ids = Vec::with_capacity(new_events.len());
            for e in new_events {
                let e = flows::create_event(&db, &mut search_engine, None, e).unwrap();
                event_ids.push(e.id);
            }
            event_ids
        };

        let mut res = client.get("/events?tag=blub&tag=bli").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));

        let mut res = client.get("/events?tag=blub").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[0])));
        assert!(body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[1])));
        assert!(!body_str.contains(&format!("<li><a href=\"/events/{}\">", event_ids[2])));
    }

    #[test]
    fn get_a_single_event() {
        let (client, db, _) = setup();
        let events = vec![Event {
            id: "1234".into(),
            title: "A great event".into(),
            description: Some("Foo bar baz".into()),
            start: NaiveDateTime::from_timestamp(0, 0),
            end: None,
            location: None,
            contact: None,
            tags: vec!["bla".into()],
            homepage: None,
            created_by: None,
            registration: Some(RegistrationType::Email),
            organizer: None,
            archived: None,
            image_url: None,
            image_link_url: None,
        }];

        {
            let db_conn = db.exclusive().unwrap();
            for e in events {
                db_conn.create_event(e).unwrap();
            }
        }

        let mut res = client.get("/events/1234").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("<h2>A great event</h2>"));
        assert!(body_str.contains("Foo bar baz</p>"));
    }
}

mod index {
    use super::*;
    #[test]
    fn get_the_index_html() {
        let (client, _db, _) = setup();
        let mut index = client.get("/").dispatch();
        assert_eq!(index.status(), Status::Ok);

        let mut index_html = client.get("/index.html").dispatch();
        assert_eq!(index_html.status(), Status::Ok);

        let index_str = index.body().and_then(|b| b.into_string()).unwrap();
        let index_html_str = index_html.body().and_then(|b| b.into_string()).unwrap();

        assert_eq!(index_html_str, index_str);
        assert!(index_str.contains("<form action=\"search\""));
        assert!(index_str.contains("<input type=\"text\""));
    }
}

mod entry {
    use super::*;
    use crate::core::usecases;
    use crate::infrastructure::flows;

    fn create_place_with_rating(
        db: &sqlite::Connections,
        search: &mut tantivy::SearchEngine,
    ) -> (String, String, String) {
        let e = usecases::NewPlace {
            title: "entry".into(),
            description: "desc".into(),
            lat: 3.7,
            lng: -50.0,
            street: None,
            zip: None,
            city: None,
            country: None,
            email: None,
            telephone: None,
            homepage: None,
            categories: vec![],
            tags: vec![],
            license: "CC0-1.0".into(),
            image_url: None,
            image_link_url: None,
        };
        let e_id = flows::prelude::create_place(db, search, e, None)
            .unwrap()
            .id;
        let r = usecases::NewPlaceRating {
            title: "A rating".into(),
            comment: "Foo".into(),
            context: RatingContext::Diversity,
            source: None,
            user: None,
            value: 1.into(),
            entry: e_id.clone().into(),
        };
        let (r_id, c_id) = flows::prelude::create_rating(db, search, r).unwrap();
        (e_id.into(), r_id, c_id)
    }

    #[test]
    fn get_entry_details() {
        let (client, db, mut search) = setup();
        let (id, _, _) = create_place_with_rating(&db, &mut search);
        let mut res = client.get(format!("/entries/{}", id)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert_eq!(body_str.contains("<form"), false);
        assert_eq!(
            body_str.contains("action=\"/comments/actions/archive\""),
            false
        );
    }

    #[test]
    fn get_entry_details_as_admin() {
        let (client, db, mut search) = setup();
        let (id, _, _) = create_place_with_rating(&db, &mut search);
        create_user(&db, "foo", Role::Admin);
        login_user(&client, "foo");
        let mut res = client.get(format!("/entries/{}", id)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert_eq!(body_str.contains("<form"), true);
        assert_eq!(
            body_str.contains("action=\"/comments/actions/archive\""),
            true
        );
    }

    #[test]
    fn get_entry_details_as_scout() {
        let (client, db, mut search) = setup();
        let (id, _, _) = create_place_with_rating(&db, &mut search);
        create_user(&db, "foo", Role::Scout);
        login_user(&client, "foo");
        let mut res = client.get(format!("/entries/{}", id)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert_eq!(body_str.contains("<form"), true);
        assert_eq!(
            body_str.contains("action=\"/comments/actions/archive\""),
            true
        );
    }

    #[test]
    fn archive_comment_as_admin() {
        let (client, db, mut search) = setup();
        create_user(&db, "foo", Role::Admin);
        login_user(&client, "foo");
        let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
        let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
        assert!(comment.archived_at.is_none());
        let res = client
            .post("/comments/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&place_id={}", c_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        //TODO: archived comments should be loaded too.
        let err = db.shared().unwrap().load_comment(&c_id).err().unwrap();
        match err {
            RepoError::NotFound => {}
            _ => panic!("Expected {}", RepoError::NotFound),
        }
    }

    #[test]
    fn archive_comment_as_scout() {
        let (client, db, mut search) = setup();
        create_user(&db, "foo", Role::Scout);
        login_user(&client, "foo");
        let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
        let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
        assert!(comment.archived_at.is_none());
        let res = client
            .post("/comments/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&place_id={}", c_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        //TODO: archived comments should be loaded too.
        let err = db.shared().unwrap().load_comment(&c_id).err().unwrap();
        match err {
            RepoError::NotFound => {}
            _ => panic!("Expected {}", RepoError::NotFound),
        }
    }

    #[test]
    fn archive_comment_as_guest() {
        let (client, db, mut search) = setup();
        let (e_id, _, c_id) = create_place_with_rating(&db, &mut search);
        let res = client
            .post("/comments/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&place_id={}", c_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);
        let comment = db.shared().unwrap().load_comment(&c_id).unwrap();
        assert!(comment.archived_at.is_none());
    }

    #[test]
    fn archive_rating_as_guest() {
        let (client, db, mut search) = setup();
        let (e_id, r_id, _) = create_place_with_rating(&db, &mut search);
        let res = client
            .post("/ratings/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&place_id={}", r_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }
}

mod admin {
    use super::*;

    #[test]
    fn change_user_role() {
        let (client, db, _) = setup();
        create_user(&db, "admin", Role::Admin);
        create_user(&db, "user", Role::User);
        let user = get_user(&db, "user");
        let admin = get_user(&db, "admin");
        assert_eq!(admin.role, Role::Admin);
        assert_eq!(user.role, Role::User);
        login_user(&client, "admin");
        let login_res = client
            .post("/change-user-role")
            .header(ContentType::Form)
            .body("email=user%40example.com&role=2")
            .dispatch();
        assert_eq!(login_res.status(), Status::SeeOther);
        let user = get_user(&db, "user");
        assert_eq!(user.role, Role::Scout);
    }
}

mod pw_reset {
    use super::*;

    #[test]
    fn reset_password() {
        let (client, db, _) = setup();
        register_user(&db, "user@example.com", "secret", true);

        // User opens the form to request a new password
        let mut res = client.get("/reset-password").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("<form"));
        assert!(body_str.contains("action=\"/users/actions/reset-password-request\""));
        assert!(body_str.contains("name=\"email\""));
        assert!(body_str.contains("type=\"submit\""));

        // User sends the request
        let res = client
            .post("/users/actions/reset-password-request")
            .header(ContentType::Form)
            .body("email=user%40example.com")
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        let h = res
            .headers()
            .iter()
            .find(|h| h.name.as_str() == "Location")
            .unwrap();
        assert_eq!(h.value, "/reset-password?success=true");

        // User gets a sucess message
        let mut res = client.get("/reset-password?success=true").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("success"));

        // User gets an email with the corresponding token
        let token = db
            .shared()
            .unwrap()
            .get_user_token_by_email("user@example.com")
            .unwrap()
            .email_nonce
            .encode_to_string();

        // User opens the link
        let mut res = client
            .get(format!("/reset-password?token={}", token))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("<form"));
        assert!(body_str.contains("action=\"/users/actions/reset-password\""));
        assert!(body_str.contains("name=\"new_password\""));
        assert!(body_str.contains("name=\"new_password_repeated\""));
        assert!(body_str.contains("name=\"token\""));
        assert!(body_str.contains("type=\"submit\""));

        // User send the new password to the server
        let res = client
            .post("/users/actions/reset-password")
            .header(ContentType::Form)
            .body(format!(
                "new_password=12345678&new_password_repeated=12345678&token={}",
                token
            ))
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        let h = res
            .headers()
            .iter()
            .find(|h| h.name.as_str() == "Location")
            .unwrap();
        assert_eq!(
            h.value,
            format!("/reset-password?token={}&success=true", token)
        );
        let mut res = client
            .get(format!("/reset-password?token={}&success=true", token))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("success"));

        // User can't login with old password
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=user%40example.com&password=secret")
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        let h = res
            .headers()
            .iter()
            .find(|h| h.name.as_str() == "Location")
            .unwrap();
        assert_eq!(h.value, "/login");

        // User can login with the new password
        let res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=user%40example.com&password=12345678")
            .dispatch();
        assert_eq!(res.status(), Status::SeeOther);
        let h = res
            .headers()
            .iter()
            .find(|h| h.name.as_str() == "Location")
            .unwrap();
        assert_eq!(h.value, "/");
    }
}
