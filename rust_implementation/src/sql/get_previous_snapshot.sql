SELECT
    snapshots.id,
    snapshots.capture_time
FROM
    snapshots
ORDER BY
    capture_time DESC
LIMIT
    1;
