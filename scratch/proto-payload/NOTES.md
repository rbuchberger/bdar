# PROTOTYPE: on-disc payload and the restore story

Throwaway. Drafts of what bdar writes onto every disc, for reacting to.
Ticket: "On-disc payload and the restore story" (rbuchberger/bdar#8).

- `README.txt` - the generic, templated README at the disc root
- `disc-layout.txt` - directory layout, volume label, collision rule, overhead
- `INDEX-sample.csv`, `FOLDERS-sample.txt`, `DISCS-sample.txt` - index formats

## Found while drafting: renames are not re-burned

Packing works on blobs with no verified copy. A rename or reorganisation creates new
`paths` rows pointing at blobs that are already safe, so nothing is burned. The
copy-every-disc-in-order restore therefore produces the *old* layout for any file moved
after its disc was burned, and a new copy of existing content never appears at its new
path. `INDEX-sample.csv` shows this as a `stored_as` column.
