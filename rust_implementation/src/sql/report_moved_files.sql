-- :snapshot_id
SELECT
    count(*),
    sum(current_files.size)
FROM
    files AS snapshot_files
    LEFT JOIN files AS current_files ON snapshot_files.hash = current_files.hash
    AND current_files.name != snapshot_files.name
    AND snapshot_files.snapshot_id = :snapshot_id
WHERE
    current_files.snapshot_id IS NULL;
