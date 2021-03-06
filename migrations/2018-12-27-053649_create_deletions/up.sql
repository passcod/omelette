CREATE TABLE deletions (
  id int GENERATED BY DEFAULT AS IDENTITY PRIMARY KEY,
  status_id int NOT NULL,
  created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
  not_before timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
  executed_at timestamp with time zone,
  sponsor text NOT NULL
);

ALTER TABLE ONLY deletions
    ADD CONSTRAINT deletions_status_id_fkey FOREIGN KEY (status_id) REFERENCES statuses(id);

COMMENT ON COLUMN deletions.id IS 'Omelette-internal ID';
COMMENT ON COLUMN deletions.status_id IS 'Omelette reference to this deletion request’s target status';
COMMENT ON COLUMN deletions.created_at IS 'When the request was created';
COMMENT ON COLUMN deletions.not_before IS 'Time before which the request must not be honored';
COMMENT ON COLUMN deletions.executed_at IS 'When Omelette executed this deletion request';
COMMENT ON COLUMN deletions.sponsor IS 'Freeform identifier for the tool that created the request';
