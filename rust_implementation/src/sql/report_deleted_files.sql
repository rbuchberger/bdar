-- :ingest_run_id (most recent)
WITH
  snapshotted AS (
    SELECT
      files.id,
      checksum,
      size,
      MAX(capture_time)
    FROM
      files
      INNER JOIN snapshots ON files.added_snapshot_id = snapshots.id
    WHERE
      deleted_snapshot_id IS NULL
    GROUP BY
      name
  ),
  -- Files from the provided ingest run (should be the most recent)
  curr_idx AS (
    SELECT
      id,
      checksum
    FROM
      files
    WHERE
      files.ingest_run_id = :ingest_run_id
  )
SELECT
  COUNT(*),
  SUM(snapshotted.size)
FROM
  snapshotted
  LEFT JOIN curr_idx ON snapshotted.checksum = curr_idx.checksum
  AND curr_idx.id != snapshotted.id
WHERE
  curr_idx.id IS NULL;
