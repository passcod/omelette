#[derive(Debug, DbEnum)]
#[PgType = "source_t"]
#[DieselType = "Source_t"]
pub enum Source {
    #[db_rename = "twitter"]
    Twitter,
    #[db_rename = "mastodon.social"]
    MastodonSocial,
}

#[derive(Debug, DbEnum)]
#[PgType = "intermediary_source_t"]
#[DieselType = "Intermediary_source_t"]
pub enum IntermediarySource {
    #[db_rename = "twitter archive"]
    TwitterArchive,
}

#[derive(Debug, DbEnum)]
#[PgType = "media_type_t"]
#[DieselType = "Media_type_t"]
pub enum MediaType {
    #[db_rename = "photo"]
    Photo,
    #[db_rename = "video"]
    Video,
    #[db_rename = "gif"]
    Gif,
}
