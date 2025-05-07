# Sqlite scratch pad

open db: `sqlite3 file.sqlite`

## run sql file

```sql
.read file.sql
```

## import csv

```sql
.import --csv file.csv table_name
```

if importing into an existing table with column mismatches, it's best to use an intermediate
"ingest" table and then:

```sql
INSERT INTO dest_table (column1, column2) 
    SELECT column1, column2 
    FROM source_table
```

## filesystem

get directory tree:
```sql
SELECT name, mtime, length(data) 
    FROM fsdir('.')
    WHERE data IS NOT NULL;
```
- fsdir takes a dir argument
- data is the actual file data. Null for directories

truncate a blob to 8 bytes: `substr(data, 1, 8)`

path parsing:

```sql
.load sqlite_extensions/path0.so 

select path_dirname(name) from fsdir(.);
```

## hash:
```sql
.load sqlite_extensions/crypto.so 
select crypto_blake3(data) from fsdir(.);
```
