-- :ingest_run_id
-- :source_dir
INSERT INTO
    files (name, modified, size, ingest_run_id)
SELECT
    name,
    mtime,
    length(data),
    :ingest_run_id
FROM
    fsdir('', :source_dir)
WHERE
    data IS NOT NULL;
