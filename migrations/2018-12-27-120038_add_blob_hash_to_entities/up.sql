ALTER TABLE entities ADD COLUMN blob_hash text;
COMMENT ON COLUMN entities.blob_hash IS 'Hash of the retrieved content in the Omelette blob store';
