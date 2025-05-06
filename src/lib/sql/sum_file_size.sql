SELECT
    SUM(size) AS bytes
FROM
    files
WHERE
    ingest_run_id = :ingest_run_id;
