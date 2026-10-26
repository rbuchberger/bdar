SELECT
    id,
    started,
    finished
FROM
    ingest_runs
ORDER BY
    started DESC
LIMIT
    2;
