INSERT INTO
    files (name, modified, size, ingest_run_id)
VALUES
    (
        :path,
        :modified,
        :size,
        :ingest_run_id
    );
