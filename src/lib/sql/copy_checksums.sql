UPDATE
    files
SET
    hash = existing.hash
FROM
    (
        SELECT
            file_id,
            hash,
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
    AND existing.hash IS NOT NULL;
