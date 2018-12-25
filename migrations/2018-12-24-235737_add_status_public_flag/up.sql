ALTER TABLE statuses ADD COLUMN public bool NOT NULL DEFAULT false;
COMMENT ON COLUMN statuses.public IS 'Whether this status was public at the time of posting, or at least at the time of fetching';
