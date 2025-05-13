# BDAR - Bare Disk Archiving Tool

- produces maximally compatible archives of large libraries, split across many disks

## Stack

- nushell
- sqlite3
- xorrisofs
- dvdisaster (CLI)
- blake3 hashing (`b3sum --raw --length 8 {filename}`)

## Directories

- user config
- database file(s)
- script file
- sqlite extensions

## DB Schema

Questions I'll want to answer:

- What files have been added, removed, and renamed in this snapshot?
- In which snapshot can I find a particular file?

### Tables

files:

- file_id
  - primary key
- added_snapshot_id
  - Snapshot where the file was added
  - index
- deleted_snapshot_id
  - Snapshot where the file disappeared
- directory
  - index
- name
  - index
  - without directory
  - Not necessarily unique
- disk_number
- modified
  - store as an integer (sqlite's unixepoch() ) which is a unix datetime. This saves a good bit of space.
- size
- unique index on snapshot_id and filename

snapshots:

- snapshot_id
- capture_time

## Operations needed

Snapshot planning:

- initialize database if required
- create new snapshot (capture current file tree)
  - exclude list
- generate a list of new/updated files
  - partition into chunks of a given size
  - just do this sequentially
- generate & burn ISO files

Snapshot application:

- generate & handle list of moved & deleted files
  - deleted: filename and hash are both missing from new snapshot
  - moved: different filename but same hash. (How to handle duplicates?)
- import new files

## Implementation

Probably a good idea to use the stor in-memory database? Will be faster, but might conflict if the
user has been using it.

### Config

- config.yml file somewhere
  - source dir
  - exclude directories
  - media size
  - ECC percentage

### Ingest

- Ingest current filesystem state
    - `INSERT INTO files name modified size SELECT name, mtime, length(data) FROM fsdir(source) WHERE data IS NOT NULL;`
- New files will have a snapshot id of null
- prune no-ops
  - join on filename, size, and mtime, delete new records when existing record matches exactly
  - new records have a snapshot_id of null
- generate hashes
  - look into running in parallel
  - do it in batches of 1000 or so
  - skip files that already have one
  - `b3sum`

### Analyze

- find new files
  - join with previous snapshot on hash, where previous value is null
  - set snapshot_id to current snapshot
  - assign disk numbers
- Find deleted files
  - set deleted_snapshot_id to current snapshot where new file id is null
- find copied & moved files
  - join new files (have current snapshot id) on hash with previous snapshot
  - If there is exactly one match, and the old file is deleted, it's a move
  - otherwise it's a copy

- produce lists of new, copied, moved, and deleted files.

- create new snapshot record
