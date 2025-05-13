-- :ingest_run_id
SELECT
    count(*),
    sum(files.size)
FROM
    files
WHERE
    ingest_run_id = :ingest_run_id
    AND hash IS NULL;
