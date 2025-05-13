SELECT
    count(*)
FROM
    disks
WHERE
    burned_timestamp IS NULL;
