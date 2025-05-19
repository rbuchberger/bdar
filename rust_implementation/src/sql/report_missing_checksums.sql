-- :ingest_run_id
SELECT
    count(*),
    sum(size)
FROM
    files
WHERE
    ingest_run_id = :ingest_run_id
    AND checksum IS NULL;
