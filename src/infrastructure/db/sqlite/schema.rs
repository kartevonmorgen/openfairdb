table! {
    bbox_subscriptions (id) {
        id -> Text,
        south_west_lat -> Double,
        south_west_lng -> Double,
        north_east_lat -> Double,
        north_east_lng -> Double,
        user_id -> Nullable<Text>,
    }
}

table! {
    categories (id) {
        id -> Text,
        created -> BigInt,
        version -> Integer,
        name -> Text,
    }
}

table! {
    comments (id) {
        id -> Text,
        created -> BigInt,
        text -> Text,
        rating_id -> Nullable<Text>,
    }
}

table! {
    entries (id, version) {
        id -> Text,
        osm_node -> Nullable<BigInt>,
        created -> BigInt,
        version -> Integer,
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
    }
}

table! {
    entry_category_relations (entry_id, entry_version, category_id) {
        entry_id -> Text,
        entry_version -> Integer,
        category_id -> Text,
    }
}

table! {
    entry_tag_relations (entry_id, entry_version, tag_id) {
        entry_id -> Text,
        entry_version -> Integer,
        tag_id -> Text,
    }
}

table! {
    ratings (id) {
        id -> Text,
        created -> BigInt,
        title -> Text,
        value -> Integer,
        context -> Text,
        source -> Nullable<Text>,
        entry_id -> Nullable<Text>,
    }
}

table! {
    tags (id) {
        id -> Text,
    }
}

table! {
    users (username) {
        id -> Text,
        username -> Text,
        password -> Text,
        email -> Text,
        email_confirmed -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    bbox_subscriptions,
    categories,
    comments,
    entries,
    entry_category_relations,
    entry_tag_relations,
    ratings,
    tags,
    users,
);
