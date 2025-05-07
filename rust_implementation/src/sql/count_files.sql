SELECT
    COUNT(*) AS count
FROM
    files
WHERE
    ingest_run_id = :ingest_run_id;
