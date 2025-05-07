-- :ingest_run_id
UPDATE
    ingest_runs
SET
    finished
VALUES
    (unixepoch())
WHERE
    id = :ingest_run_id;
