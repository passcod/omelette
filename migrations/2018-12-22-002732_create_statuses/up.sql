CREATE TYPE source_t AS ENUM (
  'twitter',
  'mastodon.social'
);

COMMENT ON TYPE source_t IS 'Sources for fetched statuses';

CREATE TYPE intermediary_source_t AS ENUM (
  'twitter archive'
);

COMMENT ON TYPE intermediary_source_t IS 'Intermediaries that retrieved a status from the original source and from which Omelette fetched from';

CREATE TABLE statuses (
  id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  text text NOT NULL,
  author_id int,
  geolocation_lat double precision,
  geolocation_lon double precision,
  posted_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
  fetched_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
  fetched_via intermediary_source_t,
  deleted_at timestamp with time zone,
  is_repost boolean DEFAULT FALSE NOT NULL,
  reposted_at timestamp with time zone,
  is_marked boolean DEFAULT FALSE NOT NULL,
  marked_at timestamp with time zone,
  source source_t NOT NULL,
  source_id text NOT NULL,
  source_author text NOT NULL,
  source_app text NOT NULL,
  in_reply_to_status text,
  in_reply_to_user text,
  quoting_status text
);

COMMENT ON COLUMN statuses.id IS 'Omelette-internal ID';
COMMENT ON COLUMN statuses.text IS 'Raw textual content of this status';
COMMENT ON COLUMN statuses.author_id IS 'Omelette reference to this status’s author';
COMMENT ON COLUMN statuses.geolocation_lat IS 'Latitude in WGS84';
COMMENT ON COLUMN statuses.geolocation_lon IS 'Longitude in WGS84';
COMMENT ON COLUMN statuses.posted_at IS 'As reported by the source';
COMMENT ON COLUMN statuses.fetched_at IS 'When Omelette retrieved this status';
COMMENT ON COLUMN statuses.fetched_via IS 'Intermediary from whom this status was retrieved';
COMMENT ON COLUMN statuses.deleted_at IS 'When Omelette deleted this status from its source';
COMMENT ON COLUMN statuses.is_repost IS 'Retweets, boosts, etc done by me';
COMMENT ON COLUMN statuses.reposted_at IS 'When this status was reposted';
COMMENT ON COLUMN statuses.is_marked IS 'Likes, favourites, etc done by me';
COMMENT ON COLUMN statuses.marked_at IS 'When this status was marked';
COMMENT ON COLUMN statuses.source IS 'Which services this status was posted to / fetched from';
COMMENT ON COLUMN statuses.source_id IS 'The opaque ID from the source';
COMMENT ON COLUMN statuses.source_author IS 'The opaque source ID of this status’s author';
COMMENT ON COLUMN statuses.source_app IS 'The opaque name and/or URL of the app from which this status was posted';
COMMENT ON COLUMN statuses.in_reply_to_status IS 'Opaque source ID of status this was in reply to';
COMMENT ON COLUMN statuses.in_reply_to_user IS 'Opaque source ID, name, or both, or list, of user this was in reply to';
COMMENT ON COLUMN statuses.quoting_status IS 'Opaque source ID of status this is quoting';
