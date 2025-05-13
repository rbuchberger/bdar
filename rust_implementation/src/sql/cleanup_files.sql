DELETE FROM
    files
WHERE
    added_snapshot_id IS NULL
    AND ingest_run_id != :ingest_run_id;
