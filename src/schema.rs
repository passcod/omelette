#![allow(proc_macro_derive_resolution_fallback)]

table! {
    use diesel::sql_types::*;
    use types::*;

    statuses (id) {
        id -> Int4,
        text -> Text,
        author_id -> Nullable<Int4>,
        geolocation_lat -> Nullable<Float8>,
        geolocation_lon -> Nullable<Float8>,
        posted_at -> Timestamptz,
        fetched_at -> Timestamptz,
        fetched_via -> Nullable<Intermediary_source_t>,
        deleted_at -> Nullable<Timestamptz>,
        is_repost -> Bool,
        reposted_at -> Nullable<Timestamptz>,
        is_marked -> Bool,
        marked_at -> Nullable<Timestamptz>,
        source -> Source_t,
        source_id -> Text,
        source_author -> Text,
        source_app -> Text,
        in_reply_to_status -> Nullable<Text>,
        in_reply_to_user -> Nullable<Text>,
        quoting_status -> Nullable<Text>,
    }
}
