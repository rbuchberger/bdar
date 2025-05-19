-- :id
-- :checksum
UPDATE
    files
SET
    checksum = :checksum
WHERE
    id = :id;
