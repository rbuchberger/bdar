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
  INNER JOIN snapshotted ON files.checksum = snapshotted.checksum
  AND files.name != snapshotted.name
WHERE
  files.added_snapshot_id IS NULL
  AND files.ingest_run_id = :ingest_run_id;
