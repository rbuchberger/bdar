-- :ingest_run_id
-- :limit
SELECT
    id,
    name,
    modified,
    size
FROM
    files
WHERE
    checksum IS NULL
    AND ingest_run_id = :ingest_run_id
LIMIT
    :limit;
