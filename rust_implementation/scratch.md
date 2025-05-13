# Scratch pad

- use jiff crate for time handling
- rustix looks interesting for handling posix system calls (specifically mounting)
- strip source dir from filename in database. Use relative paths.

## What if the filesystem changes out from underneath the database before a burn happens?

- Before passing a file to xorrisofs, we need to reindex it.
- Planning to add a copy of the database to every disk. If we ever add a setting to only add it to
one disk, it will need to be the last disk.
- Changes can only be reflected in current and later disks. Old disks may have slightly incorrect
information for the current snapshot.
- New files are ignored until the next snapshot.
- Updated files will have their metadata in the db updated to reflect the new information. The old
version is lost to history, except possibly for metadata on previous disks in the current snapshot.
- Deleted files will be marked deleted in the database. Previous disks will be incomplete.
- Moved files will be treated as deleted files in the current snapshot, and new files in the next.
This means that some space will be wasted on creating another copy that could have been avoided.

## On restore, handling directory moves

- Direct them to rmlint to get rid of empty ones. Don't bother for now.

## Multiple ingest & checksum runs

Keep database clean of old ingest runs.

- Index files in a transaction, so if it's in the database then it's complete.
- Once you've copied over checksums from previous runs that don't have snapshots, delete them.
