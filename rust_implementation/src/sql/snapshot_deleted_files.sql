-- :ingest_run_id
-- :snapshot_id
WITH
  snapshotted AS (
    SELECT
      files.id,
      checksum,
      MAX(capture_time)
    FROM
      files
      INNER JOIN snapshots ON files.added_snapshot_id = snapshots.id
    WHERE
      deleted_snapshot_id IS NULL
    GROUP BY
      name
  ),
  curr_idx AS (
    SELECT
      id,
      checksum
    FROM
      files
    WHERE
      files.ingest_run_id = :ingest_run_id
  )
UPDATE files
SET
  files.deleted_snapshot_id = :deleted_snapshot_id
WHERE
  NOT EXISTS (
    SELECT
      1
    FROM
      curr_idx
      INNER JOIN snapshotted ON curr_idx.checksum = snapshotted.checksum
      -- Checking that it's not the same record. Shouldn't be necessary because
      -- Snapshotted records will have an added_snapshot_id and the current
      -- ingest run's files should not, but that's pretty subtle behavior and I
      -- prefer to be explicit.
      AND curr_idx.id != snapshotted.id
    WHERE
      snapshotted.id = files.id
  );
