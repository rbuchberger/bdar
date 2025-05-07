-- :file_id
-- :hash
UPDATE
    files
SET
    hash = :hash
WHERE
    file_id = :file_id;
