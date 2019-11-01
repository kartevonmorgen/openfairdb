///////////////////////////////////////////////////////////////////////
// Tags
///////////////////////////////////////////////////////////////////////

table! {
    tags (id) {
        id -> Text,
    }
}


///////////////////////////////////////////////////////////////////////
// Organizations
///////////////////////////////////////////////////////////////////////

table! {
    organizations (id) {
        id -> Text,
        name -> Text,
        api_token -> Text,
    }
}

table! {
    org_tag_relations (org_id, tag_id) {
        org_id -> Text,
        tag_id -> Text,
    }
}

joinable!(org_tag_relations -> organizations (org_id));
joinable!(org_tag_relations -> tags (tag_id));


///////////////////////////////////////////////////////////////////////
// Users
///////////////////////////////////////////////////////////////////////

table! {
    users (id) {
        id -> BigInt,
        email -> Text,
        email_confirmed -> Bool,
        password -> Text,
        role -> SmallInt,
    }
}

table! {
    user_tokens (id) {
        id -> BigInt,
        user_id -> BigInt,
        expires_at -> BigInt,
        nonce -> Text,
    }
}

joinable!(user_tokens -> users (user_id));


///////////////////////////////////////////////////////////////////////
// Places
///////////////////////////////////////////////////////////////////////

table! {
    place (id) {
        id -> BigInt,
        uid -> Text,
        rev -> BigInt,
    }
}

table! {
    place_rating (id) {
        id -> BigInt,
        uid -> Text,
        place_id -> BigInt,
        created_at -> BigInt,
        created_by -> Nullable<BigInt>,
        archived_at -> Nullable<BigInt>,
        archived_by -> Nullable<BigInt>,
        title -> Text,
        value -> SmallInt,
        context -> Text,
        source -> Nullable<Text>,
    }
}

joinable!(place_rating -> place (place_id));

table! {
    place_rating_comment (id) {
        id -> BigInt,
        uid -> Text,
        rating_id -> BigInt,
        created_at -> BigInt,
        created_by -> Nullable<BigInt>,
        archived_at -> Nullable<BigInt>,
        archived_by -> Nullable<BigInt>,
        text -> Text,
    }
}

joinable!(place_rating_comment -> place_rating (rating_id));

table! {
    place_rev (id) {
        id -> BigInt,
        place_id -> BigInt,
        rev -> BigInt,
        created_at -> BigInt,
        created_by -> Nullable<BigInt>,
        status -> SmallInt,
        title -> Text,
        description -> Text,
        lat -> Double,
        lng -> Double,
        street -> Nullable<Text>,
        zip -> Nullable<Text>,
        city -> Nullable<Text>,
        country -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        homepage -> Nullable<Text>,
        license -> Nullable<Text>,
        image_url -> Nullable<Text>,
        image_link_url -> Nullable<Text>,
    }
}

joinable!(place_rev -> place (place_id));

table! {
    place_rev_status_log (id) {
        id -> BigInt,
        place_rev_id -> BigInt,
        status -> SmallInt,
        created_at -> BigInt,
        created_by -> Nullable<BigInt>,
        context -> Nullable<Text>,
        notes -> Nullable<Text>,
    }
}

joinable!(place_rev_status_log -> place_rev (place_rev_id));

table! {
    place_rev_tag (place_rev_id, tag) {
        place_rev_id -> BigInt,
        tag -> Text,
    }
}

joinable!(place_rev_tag -> place_rev (place_rev_id));


///////////////////////////////////////////////////////////////////////
// Events
///////////////////////////////////////////////////////////////////////

table! {
    events (id) {
        id -> BigInt,
        uid -> Text,
        title -> Text,
        description -> Nullable<Text>,
        start -> BigInt,
        end -> Nullable<BigInt>,
        lat -> Nullable<Double>,
        lng -> Nullable<Double>,
        street -> Nullable<Text>,
        zip -> Nullable<Text>,
        city -> Nullable<Text>,
        country -> Nullable<Text>,
        email -> Nullable<Text>,
        telephone -> Nullable<Text>,
        homepage -> Nullable<Text>,
        created_by -> Nullable<BigInt>,
        registration -> Nullable<SmallInt>,
        organizer -> Nullable<Text>,
        archived -> Nullable<BigInt>,
        image_url -> Nullable<Text>,
        image_link_url -> Nullable<Text>,
    }
}

joinable!(events -> users (created_by));

table! {
    event_tags (event_id, tag) {
        event_id -> BigInt,
        tag -> Text,
    }
}

joinable!(event_tags -> events (event_id));


///////////////////////////////////////////////////////////////////////
// Subscriptions
///////////////////////////////////////////////////////////////////////

table! {
    bbox_subscriptions (id) {
        id -> BigInt,
        uid -> Text,
        user_id -> BigInt,
        south_west_lat -> Double,
        south_west_lng -> Double,
        north_east_lat -> Double,
        north_east_lng -> Double,
    }
}

joinable!(bbox_subscriptions -> users (user_id));


///////////////////////////////////////////////////////////////////////

allow_tables_to_appear_in_same_query!(
    bbox_subscriptions,
    events,
    event_tags,
    place,
    place_rating,
    place_rating_comment,
    place_rev,
    place_rev_status_log,
    place_rev_tag,
    org_tag_relations,
    organizations,
    tags,
    users,
    user_tokens,
);
