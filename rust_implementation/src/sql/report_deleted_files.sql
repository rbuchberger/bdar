-- :snapshot_id
SELECT
    count(*),
    sum(snapshot_files.size)
FROM
    files AS snapshot_files
    LEFT JOIN files AS current_files ON snapshot_files.hash = current_files.hash
    AND snapshot_files.snapshot_id = :snapshot_id
    AND current_files.snapshot_id IS NULL
WHERE
    current_files.id IS NULL;
