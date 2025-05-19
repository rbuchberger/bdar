UPDATE
    files
SET
    checksum = existing.checksum
FROM
    (
        SELECT
            id,
            checksum,
            modified,
            name,
            size
        FROM
            files
    ) AS existing
WHERE
    files.ingest_run_id = :ingest_run_id
    AND files.modified = existing.modified
    AND files.size = existing.size
    AND files.name = existing.name
    AND existing.checksum IS NOT NULL;
