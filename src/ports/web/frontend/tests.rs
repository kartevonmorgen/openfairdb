use super::*;
use crate::infrastructure::db::tantivy;
use crate::ports::web::tests::prelude::*;

fn setup() -> (
    rocket::local::Client,
    sqlite::Connections,
    tantivy::SearchEngine,
) {
    crate::ports::web::tests::setup(vec![("/", super::routes())])
}

mod events {
    use super::*;

    #[test]
    fn get_a_list_of_all_events() {
        let (client, db, _) = setup();
        let events = vec![
            Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::hours(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
            Event {
                id: "5678".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::days(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
            Event {
                id: "0000".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::days(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
        ];

        {
            let db_conn = db.exclusive().unwrap();
            for e in events {
                db_conn.create_event(e).unwrap();
            }
        }

        let mut res = client.get("/events").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(body_str.contains("<li><a href=\"/events/1234\">"));
        assert!(body_str.contains("<li><a href=\"/events/5678\">"));
        assert!(!body_str.contains("<li><a href=\"/events/0000\">"));
    }

    #[test]
    fn get_a_list_of_events_filtered_by_tags() {
        let (client, db, _) = setup();
        let events = vec![
            Event {
                id: "1234".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::hours(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bla".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
            Event {
                id: "5678".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::days(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["bli".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
            Event {
                id: "0000".into(),
                title: "x".into(),
                description: None,
                start: chrono::Utc::now()
                    .checked_sub_signed(chrono::Duration::days(2))
                    .unwrap()
                    .naive_utc(),
                end: None,
                location: None,
                contact: None,
                tags: vec!["blub".into()],
                homepage: None,
                created_by: None,
                registration: Some(RegistrationType::Email),
                organizer: None,
                archived: None,
            },
        ];

        {
            let db_conn = db.exclusive().unwrap();
            for e in events {
                db_conn.create_event(e).unwrap();
            }
        }

        let mut res = client.get("/events?tag=blub&tag=bli").dispatch();
        assert_eq!(res.status(), Status::Ok);
        let body_str = res.body().and_then(|b| b.into_string()).unwrap();
        assert!(!body_str.contains("<li><a href=\"/events/1234\">"));
        assert!(body_str.contains("<li><a href=\"/events/5678\">"));
        // '0000' has "blub" but its too old
        assert!(!body_str.contains("<li><a href=\"/events/0000\">"));
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

    fn create_entry_with_rating(
        db: &sqlite::Connections,
        search: &mut tantivy::SearchEngine,
    ) -> (String, String, String) {
        let e = usecases::NewEntry {
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
        let e_id = flows::prelude::create_entry(db, search, e).unwrap();
        let r = usecases::RateEntry {
            title: "A rating".into(),
            comment: "Foo".into(),
            context: RatingContext::Diversity,
            source: None,
            user: None,
            value: 1.into(),
            entry: e_id.clone(),
        };
        let (r_id, c_id) = flows::prelude::create_rating(db, search, r).unwrap();
        (e_id, r_id, c_id)
    }

    #[test]
    fn get_entry_details() {
        let (client, db, mut search) = setup();
        let (id, _, _) = create_entry_with_rating(&db, &mut search);
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
        let (id, _, _) = create_entry_with_rating(&db, &mut search);
        super::super::login::tests::register_user(&db, "foo@bar.com", "bazbaz", true);
        let mut user = db
            .shared()
            .unwrap()
            .get_users_by_email("foo@bar.com")
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        user.role = Role::Admin;
        db.exclusive().unwrap().update_user(&user).unwrap();
        let login_res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=foo%40bar.com&password=bazbaz")
            .dispatch();
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
    fn archive_comment_as_guest() {
        let (client, db, mut search) = setup();
        let (e_id, r_id, c_id) = create_entry_with_rating(&db, &mut search);
        let res = client
            .post("/comments/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&entry_id={}", c_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }

    #[test]
    fn archive_rating_as_guest() {
        let (client, db, mut search) = setup();
        let (e_id, r_id, c_id) = create_entry_with_rating(&db, &mut search);
        let res = client
            .post("/ratings/actions/archive")
            .header(ContentType::Form)
            .body(format!("ids={}&entry_id={}", r_id, e_id))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);
    }
}

mod admin {
    use super::*;
    use crate::core::usecases;
    use crate::infrastructure::flows;

    #[test]
    fn change_user_role() {
        let (client, db, mut search) = setup();
        super::super::login::tests::register_user(&db, "admin@example.com", "secret", true);
        super::super::login::tests::register_user(&db, "user@example.com", "secret", true);
        let mut admin = db
            .shared()
            .unwrap()
            .get_users_by_email("admin@example.com")
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        admin.role = Role::Admin;
        db.exclusive().unwrap().update_user(&admin).unwrap();
        {
            let db = db.shared().unwrap();
            let user = db
                .get_users_by_email("user@example.com")
                .unwrap()
                .into_iter()
                .next()
                .unwrap();
            let admin = db
                .get_users_by_email("admin@example.com")
                .unwrap()
                .into_iter()
                .next()
                .unwrap();
            assert_eq!(admin.role, Role::Admin);
            assert_eq!(user.role, Role::User);
        }
        let login_res = client
            .post("/login")
            .header(ContentType::Form)
            .body("email=admin%40example.com&password=secret")
            .dispatch();
        let login_res = client
            .post("/change-user-role")
            .header(ContentType::Form)
            .body("email=user%40example.com&role=2")
            .dispatch();
        let user = db
            .shared()
            .unwrap()
            .get_users_by_email("user@example.com")
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(user.role, Role::Scout);
    }
}
