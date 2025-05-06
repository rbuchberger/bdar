-- :ingest_run_id
SELECT
    name,
    file_id
FROM
    files
WHERE
    hash IS NULL
    AND ingest_run_id = :ingest_run_id
LIMIT
    100
