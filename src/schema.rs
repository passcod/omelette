// This file is auto-generated by diesel. For hand edits, see the patch file.
#![allow(proc_macro_derive_resolution_fallback)]

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    entities (id) {
        id -> Int4,
        fetched_at -> Timestamptz,
        status_id -> Int4,
        ordering -> Nullable<Int4>,
        media_type -> Media_type_t,
        source_id -> Text,
        source_url -> Text,
        original_status_source_id -> Nullable<Text>,
        original_status_source_url -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::types::*;

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
        public -> Bool,
    }
}

joinable!(entities -> statuses (status_id));

allow_tables_to_appear_in_same_query!(
    entities,
    statuses,
);
