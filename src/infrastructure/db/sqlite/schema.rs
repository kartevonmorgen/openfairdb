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

table! {
    categories (id) {
        id -> Text,
        created -> BigInt,
        version -> BigInt,
        name -> Text,
    }
}

table! {
    comments (id) {
        id -> Text,
        created -> BigInt,
        archived -> Nullable<BigInt>,
        text -> Text,
        rating_id -> Text,
    }
}

table! {
    entries (id, version) {
        id -> Text,
        osm_node -> Nullable<BigInt>,
        created -> BigInt,
        archived -> Nullable<BigInt>,
        version -> BigInt,
        current -> Bool,
        title -> Text,
        description -> Text,
        lat -> Double,
        lng -> Double,
        street -> Nullable<Text>,
        zip -> Nullable<Text>,
        city -> Nullable<Text>,
        country -> Nullable<Text>,
        email -> Nullable<Text>,
        telephone -> Nullable<Text>,
        homepage -> Nullable<Text>,
        license -> Nullable<Text>,
        image_url -> Nullable<Text>,
        image_link_url -> Nullable<Text>,
    }
}

table! {
    entry_category_relations (entry_id, entry_version, category_id) {
        entry_id -> Text,
        entry_version -> BigInt,
        category_id -> Text,
    }
}

table! {
    entry_tag_relations (entry_id, entry_version, tag_id) {
        entry_id -> Text,
        entry_version -> BigInt,
        tag_id -> Text,
    }
}

table! {
    event_tags (event_id, tag) {
        event_id -> BigInt,
        tag -> Text,
    }
}

table! {
    org_tag_relations (org_id, tag_id) {
        org_id -> Text,
        tag_id -> Text,
    }
}

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

table! {
    ratings (id) {
        id -> Text,
        created -> BigInt,
        archived -> Nullable<BigInt>,
        title -> Text,
        value -> Integer,
        context -> Text,
        source -> Nullable<Text>,
        entry_id -> Text,
    }
}

table! {
    tags (id) {
        id -> Text,
    }
}

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

table! {
    organizations (id) {
        id -> Text,
        name -> Text,
        api_token -> Text,
    }
}

joinable!(bbox_subscriptions -> users (user_id));
joinable!(comments -> ratings (rating_id));
joinable!(entry_category_relations -> categories (category_id));
joinable!(entry_tag_relations -> tags (tag_id));
joinable!(event_tags -> events (event_id));
joinable!(events -> users (created_by));
joinable!(org_tag_relations -> organizations (org_id));
joinable!(org_tag_relations -> tags (tag_id));
joinable!(user_tokens -> users (user_id));

allow_tables_to_appear_in_same_query!(events, event_tags,);

allow_tables_to_appear_in_same_query!(
    bbox_subscriptions,
    categories,
    comments,
    entries,
    entry_category_relations,
    entry_tag_relations,
    events,
    org_tag_relations,
    organizations,
    ratings,
    tags,
    users,
    user_tokens,
);
