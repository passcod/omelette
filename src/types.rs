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
