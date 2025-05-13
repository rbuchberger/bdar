SELECT
    count(*),
    sum(files.size)
FROM
    files
WHERE
    added_snapshot_id IS NULL;
