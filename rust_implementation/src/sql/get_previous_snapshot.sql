SELECT
    snapshots.id,
    snapshots.capture_time,
    count(files.id),
    sum(files.size)
FROM
    snapshots
    LEFT JOIN files ON files.added_snapshot_id = snapshots.id
GROUP BY
    snapshots.id,
    capture_time
ORDER BY
    capture_time DESC
LIMIT
    1;
