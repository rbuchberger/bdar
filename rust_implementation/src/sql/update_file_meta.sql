UPDATE
    files
SET
    size = :size,
    modified = :modified
WHERE
    id = :id;
