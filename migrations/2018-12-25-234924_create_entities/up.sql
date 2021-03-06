CREATE TYPE media_type_t AS ENUM (
  'photo',
  'video',
  'gif'
);

COMMENT ON TYPE media_type_t IS 'Types of media entities attached to statuses';

CREATE TABLE entities (
  id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  fetched_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
  status_id int NOT NULL,
  ordering int,
  media_type media_type_t NOT NULL,
  source_id text NOT NULL,
  source_url text NOT NULL,
  original_status_source_id text,
  original_status_source_url text
);

ALTER TABLE ONLY entities
    ADD CONSTRAINT entities_status_id_fkey FOREIGN KEY (status_id) REFERENCES statuses(id);

COMMENT ON COLUMN entities.id IS 'Omelette-internal ID';
COMMENT ON COLUMN entities.fetched_at IS 'When Omelette retrieved this entity';
COMMENT ON COLUMN entities.status_id IS 'Omelette reference to this entity’s status';
COMMENT ON COLUMN entities.ordering IS 'Ordering of this entity in relation to other entities attached to a status';
COMMENT ON COLUMN entities.media_type IS 'Type of the entity';
COMMENT ON COLUMN entities.source_id IS 'The opaque ID from the source';
COMMENT ON COLUMN entities.source_url IS 'The opaque source URL containing the actual media';
COMMENT ON COLUMN entities.original_status_source_id IS 'If the entity was originally attached to a different status we pulled it from, this will be its source ID';
COMMENT ON COLUMN entities.original_status_source_url IS 'The original source URL to the source-framed entity display';
