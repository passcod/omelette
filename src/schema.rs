// This file is auto-generated by diesel. For hand edits, see the patch file.
#![allow(proc_macro_derive_resolution_fallback)]

table! {
    use diesel::sql_types::*;

    deletions (id) {
        id -> Int4,
        status_id -> Int4,
        created_at -> Timestamptz,
        not_before -> Timestamptz,
        executed_at -> Nullable<Timestamptz>,
        sponsor -> Text,
    }
}

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
        blob_hash -> Nullable<Text>,
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

table! {
    use diesel::sql_types::*;
    use crate::types::*;

    twitter_users (id) {
        id -> Int4,
        source_id -> Text,
        screen_name -> Text,
        name -> Text,
        description -> Nullable<Text>,
        location -> Nullable<Text>,
        url -> Nullable<Text>,
        is_verified -> Bool,
        is_protected -> Bool,
        is_coauthored -> Bool,
        is_translator -> Bool,
        statuses_count -> Int4,
        following_count -> Int4,
        followers_count -> Int4,
        likes_count -> Int4,
        listed_count -> Int4,
        created_at -> Timestamptz,
        fetched_at -> Timestamptz,
        blocked_at -> Nullable<Timestamptz>,
        muted_at -> Nullable<Timestamptz>,
        missing -> Bool,
        ui_language -> Nullable<Text>,
        ui_timezone -> Nullable<Text>,
        withheld_in -> Nullable<Text>,
        withheld_scope -> Nullable<Text>,
    }
}

joinable!(deletions -> statuses (status_id));
joinable!(entities -> statuses (status_id));

allow_tables_to_appear_in_same_query!(
    deletions,
    entities,
    statuses,
    twitter_users,
);
