-- :ingest_run_id
-- :previous_snapshot_id
DELETE FROM
    files
WHERE
    files.file_id IN (
        SELECT
            files.file_id
        FROM
            files
            INNER JOIN files AS existing ON files.name = existing.name
            AND files.hash = existing.hash
            AND existing.snapshot_id IS NOT NULL
            AND existing.hash IS NOT NULL
        WHERE
            existing.snapshot_id = :previous_snapshot_id
    )
