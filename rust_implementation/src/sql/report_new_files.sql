SELECT
    count(*),
    sum(current_files.size)
FROM
    files AS current_files
    LEFT JOIN files AS snapshot_files ON snapshot_files.name = current_files.name
    AND snapshot_files.added_snapshot_id IN (
        -- most recent snapshot
        SELECT
            id
        FROM
            snapshots
        ORDER BY
            capture_time DESC
        LIMIT
            1
    )
WHERE
    current_files.added_snapshot_id IS NULL
    AND snapshot_files.id IS NULL;
