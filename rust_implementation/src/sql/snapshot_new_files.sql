-- Includes moved files.
-- :ingest_run_id
-- :snapshot_id
WITH
  snapshotted AS (
    SELECT
      checksum,
      MAX(capture_time)
    FROM
      files
      INNER JOIN snapshots ON files.added_snapshot_id = snapshots.id
    GROUP BY
      name
  )
UPDATE files
SET
  added_snapshot_id = :snapshot_id
WHERE
  added_snapshot_id IS NULL
  AND ingest_run_id = :ingest_run_id
  AND NOT EXISTS (
    SELECT
      1
    FROM
      snapshotted
    WHERE
      snapshotted.checksum = files.checksum
  );
