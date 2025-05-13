SELECT
    count(*),
    sum(files.size)
FROM
    files
WHERE
    files.ingest_run_id = :ingest_run_id;
