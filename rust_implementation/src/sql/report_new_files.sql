-- Does not include moved files.
-- :ingest_run_id
WITH
  snapshotted AS (
    SELECT
      name,
      checksum,
      MAX(capture_time)
    FROM
      files
      INNER JOIN snapshots ON files.added_snapshot_id = snapshots.id
    GROUP BY
      name
  )
SELECT
  COUNT(*),
  SUM(files.size)
FROM
  files
  LEFT JOIN snapshotted ON snapshotted.name = files.name
  AND files.checksum = snapshotted.checksum
WHERE
  files.added_snapshot_id IS NULL
  AND files.ingest_run_id = :ingest_run_id
  AND snapshotted.name IS NULL;
