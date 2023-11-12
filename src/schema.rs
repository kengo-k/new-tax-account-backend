// @generated automatically by Diesel CLI.

diesel::table! {
    category (id) {
        id -> Nullable<Integer>,
        name -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    posts (id) {
        id -> Nullable<Integer>,
        title -> Text,
        body -> Text,
        category_id -> Nullable<Integer>,
        author -> Nullable<Text>,
        published -> Bool,
        good_count -> Integer,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(category, posts,);
