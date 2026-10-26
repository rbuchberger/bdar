# bdar design spec

The locked design for bdar, consolidated from the wayfinder map
([#1](https://github.com/rbuchberger/bdar/issues/1)) and its tickets, with every amendment applied
flat. Terms are those in `GLOSSARY.md`. Decisions here are settled; a ticket that contradicts one
must say so explicitly.

Established research facts (capacities, tool behaviour, hardware) are in the map's "Established
facts" section and are not repeated.

## Problem Statement

I have a large photo library (terabytes, hundreds of thousands of files) that I want on
write-once optical media as a backup of last resort: ransomware-proof, immune to backup
misconfiguration, and readable decades from now. The library grows and gets reorganised every
year, so the backup has to be incremental, and the whole thing has to be burned over weeks of
disc swaps without that being fragile.

The people most likely to need the backup are not me. They have a Windows PC and no software,
no command line, and no instructions beyond what is on the discs. Existing tools (`dar`) solve
the splitting and incremental problem but produce discs nobody can read without an explanation.

## Solution

bdar writes bare files to discs: real folders and files at their real relative paths, readable
by putting a disc in any PC. It keeps a small database describing every file and the disc holding
it, and puts a copy of that database, a plain-text index, a README and the dvdisaster repair tool
onto every disc, so the newest disc alone restores the whole archive.

It drives the whole pipeline: take a snapshot of the library, plan one disc at a time, build an
ISO, add error-correction data, burn, read the disc back and verify it, record the result, and
move to the next blank. The same code path handles the initial bulk load and the handful of discs
a year after that. A half-burned snapshot set is a normal, durable state. Discs are scrubbed
periodically; a disc that degrades is retired and its content burned again onto new discs.

## User Stories

Actors: the **archivist** (owns the library and the burner), the **restorer** (non-technical,
Windows, has only the discs), the **helper** (technical person assisting a restorer), another
**user** adopting bdar for a different library or medium, and the **developer**.

### Defining the archive

1. As an archivist, I want to name the directory trees to archive as an explicit allowlist, so
   that a junk directory appearing next to them is never silently swept in.
2. As an archivist, I want one global exclude list of glob patterns, so that the definition of
   the backup set fits on one screen and can be audited.
3. As an archivist, I want to declare a `prepare` command on a root, so that generated or rotating
   content (a database dump, a compose file) is staged into an ordinary directory before the walk.
4. As an archivist, I want the on-disc layout to mirror the source tree relative to a frozen
   archive prefix, so that a restorer can copy a disc back into place.
5. As an archivist, I want bdar to refuse a root outside the frozen prefix, so that a config edit
   can never silently relocate every future disc's layout.
6. As an archivist, I want a `disc_extras` directory copied onto every disc, so that the newest
   disc explains the library itself, not just the archive.
7. As an archivist, I want secrets to stay off the discs, so that a box of media in a cupboard is
   not a credential leak.
8. As an archivist, I want several archives in one config file, so that a throwaway test archive
   and the real one coexist without duplicating the tool templates.

### Taking snapshots

9. As an archivist, I want `bdar snapshot` to walk the roots and record every file's content hash,
   so that bdar knows what exists and what is already on a disc.
10. As an archivist, I want a snapshot to abort with nothing written if a root is unreachable, a
    `prepare` fails, a file is unreadable, or a symlink dangles or loops, so that a partial walk is
    never mistaken for a mass deletion.
11. As an archivist, I want size and mtime to skip re-hashing unchanged files, so that a yearly
    snapshot of terabytes takes minutes, not hours.
12. As an archivist, I want `--rehash` to hash everything, so that I can catch an edit that kept
    size and mtime.
13. As an archivist, I want an interrupted first snapshot to resume without re-hashing what it
    already did, so that Ctrl-C during a multi-hour walk costs at most a batch.
14. As an archivist, I want symlinks followed, hardlinks treated as ordinary files, and zero-byte
    files preserved, so that the disc reproduces what was there. Empty directories are not kept.
15. As an archivist, I want `snapshot --dry-run` to tell me what changed and how much needs
    burning, so that I can decide whether to start a set.
16. As an archivist, I want a warning for file names that differ only by case in one folder, so
    that I know Windows will only open one of them.
17. As an archivist, I want a warning for path components longer than Joliet allows, so that I
    know Windows will show a different name.
18. As an archivist, I want taking a new snapshot to supersede the old one and carry its unburned
    work forward, so that abandoning a set and folding in new photos are the same simple act.
19. As an archivist, I want `snapshot` to refuse while a disc is in flight, so that a disc never
    belongs to a superseded snapshot.
20. As an archivist, I want a moved file to cost nothing to burn, so that reorganising the library
    doesn't mean burning it again.

### Freezing the source

21. As an archivist with a snapshotting filesystem, I want to give bdar `freeze` and `release`
    commands and a frozen path, so that every disc of a set is built from one consistent tree.
22. As an archivist, I want bdar to derive the freeze name from the snapshot, so that after a
    reboot nothing has to be searched for.
23. As an archivist, I want the freeze released only when the set is complete or superseded, so
    that it is never destroyed while a disc still needs it.
24. As an archivist, I want a failed release to warn and never block a burn.
25. As an archivist, I want `burn` to fail, not fall back to the live tree, when the frozen tree
    is gone, so that a disc never silently contains different bytes than the snapshot recorded.
26. As a user without a snapshotting filesystem, I want bdar to work on the live tree with the
    same archive correctness, so that ZFS is a recommendation and not a requirement.

### Burning

27. As an archivist, I want `bdar burn` to plan the next disc, build it, burn it, verify it and
    move on to the next blank in a loop, so that burning sixty discs is sixty swaps and nothing
    else.
28. As an archivist, I want each disc planned only when a blank is in the drive, against that
    blank's media profile, so that one set may mix BD-R DL, BD-R SL and DVD.
29. As an archivist, I want discs packed in path order with no gap filling, so that each disc is
    one contiguous range of the tree and a lost disc costs a range, not a scatter.
30. As an archivist, I want a blob's paths never split across discs and identical files stored
    once per disc, so that duplicate copies in the library cost nothing extra.
31. As an archivist, I want each disc to land as close to the payload ceiling as possible on the
    first sizing call, so that media isn't wasted and planning isn't slow.
32. As an archivist, I want a file larger than the medium skipped and reported, never split and
    never a crash, so that one big video doesn't block the set.
33. As an archivist, I want to be asked before a partial last disc is burned, so that I can wait
    for more content instead of spending a blank on a quarter-full disc.
34. As an archivist, I want the staged ISO checked against the recorded hashes and mtimes before
    any media is spent, so that a bad build or a file that changed mid-build is caught for free.
35. As an archivist, I want the burned disc read back and hashed, every sector including ECC, so
    that "verified" means the disc in my hand matches the image.
36. As an archivist, I want the drive cache defeated before the read-back, with bdar telling me
    to leave the disc in, so that verification reads the medium and not RAM.
37. As an archivist, I want a disc to count as archived only once it is verified, so that the
    database never describes a disc that doesn't exist.
38. As an archivist, I want a failed burn to keep the disc number and the staged ISO and retry on
    a fresh blank, so that a coaster costs one blank and nothing else.
39. As an archivist, I want every burn attempt kept as a row with its outcome and log file, so
    that a coaster can be diagnosed later.
40. As an archivist, I want `burn` to resume from whatever it finds (partial ISO, staged ISO,
    burned-but-unverified disc) with no separate recovery command, so that a crash or a
    week's gap is nothing special.
41. As an archivist, I want `burn` to probe the disc in the drive rather than assume, so that it
    never writes onto a disc that already has data.
42. As an archivist, I want a single Ctrl-C during a write to warn and a second within ten seconds
    to abort, so that a stray keypress doesn't make a coaster.
43. As an archivist, I want a dropped SSH session to leave the burn running, so that a network
    blip doesn't waste a disc.
44. As an archivist, I want a verification I/O error to be retryable by running `burn` again, and
    a clean read with the wrong hash to be a coaster, so that the two failures are handled
    differently.
45. As an archivist, I want a per-attempt log file with every command, its output, exit code and
    timings, so that a coaster can be diagnosed.
46. As an archivist, I want `burn --discard` to drop an in-flight disc and free its number, so
    that I can get out of a bad state.
47. As an archivist, I want `burn --dry-run` to show what goes on the next disc, so that I can
    sanity-check a set before spending media.
48. As an archivist, I want a space check before anything is created, so that a build never
    fails forty gigabytes in.
49. As an archivist, I want a file that changed after the snapshot to stop the burn before media
    is spent, with a pointer to re-snapshot, so that a disc never carries bytes the db doesn't
    describe.
50. As an archivist, I want the burn tool's own progress passed through live, so that I can see a
    forty-minute write is moving.

### Scrubbing and retiring

51. As an archivist, I want `bdar scrub` to read every sector of every disc I feed it and record
    the result, so that a degraded disc is found before its data is needed.
52. As an archivist, I want `status` to list discs overdue for a scrub, oldest first, so that an
    evening a year keeps the archive checked.
53. As an archivist, I want `scrub` to identify each disc from its label and refuse another
    archive's disc, so that I can feed it a box of discs in any order.
54. As an archivist, I want a degraded disc retired, its live content returning to remaining
    work and burned onto new discs by the normal planner, so that remaking needs no new pipeline.
55. As an archivist, I want `bdar retire <disc> --reason lost|unreadable|degraded`, so that a
    lost or unreadable disc is handled the same way.
56. As an archivist, I want retirement to be final and stored, so that a later clean read can't
    make a decaying disc's copies count again.

### Reading the archive

57. As an archivist, I want `bdar find <pattern>` to tell me which disc holds a file, including as
    of a past date, so that one file comes back without touching sixty discs.
58. As an archivist, I want `bdar status` to summarise the archive: latest snapshot, remaining
    work, disc in flight, warnings, so that I know what to do next.
59. As an archivist, I want `--json` on read commands and run summaries, so that I can pipe
    them into my shell.
60. As a helper, I want `bdar --db <path>` to open a disc's database copy read-only, so that
    `status` and `find` work from a mounted disc on a machine with no config.

### Restoring

61. As a restorer, I want to put a disc in a Windows PC and see folders and photos, so that I
    need no software and no instructions.
62. As a restorer, I want a `README.txt` on every disc that says, in plain words, how to get
    everything back (copy every disc into one folder in order, say yes to overwrite) and how to get
    one folder back, so that I can do it alone.
63. As a restorer, I want a plain-text `FOLDERS.txt` saying which discs hold each folder, so that
    I can find a year's photos without opening a database.
64. As a restorer, I want the README to warn me that files renamed after they were burned are on
    disc under their old name, and that copied-off discs take more space than the disc shows, so
    that I'm not confused by what I see.
65. As a restorer, I want the README to walk me through repairing a disc that won't read, using
    the dvdisaster program copied off another disc, on Windows or Linux, so that a bad disc isn't
    the end.
66. As a helper, I want the full database and `INDEX.csv` on the newest disc, so that I can answer
    any question about the archive's history with only that disc.
67. As a helper, I want the README to show the dvdisaster command that reads the whole device,
    so that I don't trim the ECC off while trying to recover a disc.
68. As a helper, I want `DISCS.txt` to mark retired discs and say their content is on later
    discs, so that I don't chase a dead disc.

### Configuration and other media

69. As a user, I want bdar to run with no config file at all, so that the shipped defaults are the
    program and not an example to copy.
70. As a user, I want to override any tool's command line as an argv array, so that a different
    burner, a different medium or a different version of a tool needs no bdar release.
71. As a user, I want built-in media profiles for BD-R DL, BD-R SL and DVD and the ability to
    define my own with its own payload ceiling and tool overrides, so that an unforeseen medium is
    a config change.
72. As a user, I want `bdar config dump` to print the effective config and `--defaults` the
    shipped one, so that I can diff after an upgrade without a stale copy.
73. As a user, I want a warning when a shipped default I forked from has since changed, so that
    my override doesn't go stale silently.
74. As a user, I want a load-time error for a malformed template, unknown step or unresolvable
    placeholder, so that a config mistake fails before a burn, not during one.
75. As a user, I want an advisory warning when a template omits a known-destructive flag, so
    that I'm told without bdar pretending to understand another tool's arguments.
76. As a user, I want the correctness checks to be bdar's own code and not a template, so that I
    cannot weaken them by editing a string.
77. As a user, I want to disable ECC by emptying a template, so that "no ECC" needs no special
    case.
78. As a user, I want `burn` to refuse a real archive when the dvdisaster zips aren't where it
    expects, telling me the path and the download URLs, so that no disc leaves without its repair
    tool.

### Development and rehearsal

79. As a developer, I want a test archive whose device is a regular file, so that the whole
    pipeline runs to `verified` on the real code path with no drive.
80. As a developer, I want a file device refused outside a `test = true` archive, so that a
    misconfigured real archive can't report blobs as safe while they sit on an SSD.
81. As a developer, I want table-driven corpus tests written as sequences of library states, so
    that every dedup and deletion case has one definition.
82. As a developer, I want the default `cargo test` to need only xorriso and run in CI, with
    tool-dependent tests opt-in and failing loudly when a tool is missing.
83. As an archivist, I want a written, rerunnable DVD rehearsal, so that the pipeline is proven on
    real hardware before the first expensive disc.
84. As an archivist, I want the first real disc to be a gate with a pause to inspect and time it,
    so that the bulk load is planned from a measurement.

## Implementation Decisions

### Codebase

- Keep the existing Rust crate and rewrite `src/` and the schema in place; the crate moves from
  `rust_implementation/` to the repo root. Delete the nushell implementation. Keep the on-disc
  info material (dvdisaster binaries and manual) as reference.
- Dependencies already settled: `rusqlite` (bundled), `blake3` crate (in-process; never `b3sum`),
  `clap`, `walkdir`, TOML config, `serde` for `--json`. YAML is dropped.
- Linux only. An ordinary dynamically linked binary; nothing bdar builds goes on a disc.
- License GPL-3.0.

### Configuration

TOML at `$XDG_CONFIG_HOME/bdar/config.toml`, overridden by `--config`. Missing file means
compiled-in defaults. The file is a sparse overlay: only overrides are stated. `bdar config dump`
prints the effective config; `--defaults` prints the shipped one, with `forked_from` filled in so
copying a default pastes it.

Global keys:

- `device` — the drive, e.g. `/dev/sr0`; `--device` overrides. May be a regular file only in a
  test archive.
- `auto_reload` (default `false`) — skip the confirmation before the eject/reload cycle.
- `disc_tools_dir` (default `/usr/share/bdar/tools`) — holds the two dvdisaster zips (Windows
  GUI, Linux static CLI) that go on every disc.
- `[tools.<step>]` — five fixed step names: `iso_size`, `mkiso`, `ecc`, `burn`, `probe`. Unknown
  name is a load error. Each has `argv` (array), optional `ack_warnings` (silence the advisory
  flag warnings for that step), optional `forked_from` (bdar version the override was copied
  from). Empty `argv` disables the step; `iso_size`, `mkiso`, `burn` and `probe` are required, so
  empty there is a load error. `iso_size` additionally has `stdout_is = "sectors"` (last integer
  on stdout, trailing `s` accepted). `probe`'s output contract is "cdrskin `-minfo` compatible",
  documented.
- `[media_profiles.<name>]` — `capacity_bytes`, `max_payload_bytes` (the packing ceiling; the
  packer uses only this and never reasons about ECC), and any other keys, exposed to templates as
  placeholders. A profile may contain `[media_profiles.<name>.tools.<step>]` overriding a step
  wholesale. Built-ins: `bdr-dl`, `bdr-sl`, `dvd`.
- `[archives.<name>]` — `exclude` (global list of globs), `media_profile` (default profile),
  `db` (path; default `$XDG_DATA_HOME/bdar/<archive>.sqlite`), `staging_dir` (default
  `$XDG_CACHE_HOME/bdar/<archive>/`), `disc_extras` (directory path), `short_name` (≤ 12 chars,
  used in volume labels), `test` (bool), `scrub_interval` (days, default 365), `frozen_root`
  (path template using `{freeze_name}`), and hook templates `freeze`, `release` (argv).
  `short_name` and `test` are frozen in the db on the first snapshot; config disagreeing with the
  db is a load error.
- `[[archives.<name>.root]]` — one table per root: `path`, optional `prepare` (argv). No name;
  nothing reads one.

```toml
[[archives.photos.root]]
path = "/library/originals"

[[archives.photos.root]]
path    = "/library/scans"
prepare = ["rsync", "-a", "--delete", "/mnt/scanner/out/", "{root}"]
```

Shipped defaults:

```toml
[tools.iso_size]
argv = ["xorriso", "-no_rc", "-outdev", "stdio:{iso_path}", "-volid", "{volume_label}",
        "-joliet", "on", "-compliance", "iso_9660_level=3:joliet_long_names",
        "-options_from_file", "{iso_commands}", "-print_size", "-rollback_end"]
stdout_is = "sectors"

[tools.mkiso]
argv = ["xorriso", "-no_rc", "-outdev", "stdio:{iso_path}", "-volid", "{volume_label}",
        "-joliet", "on", "-compliance", "iso_9660_level=3:joliet_long_names",
        "-options_from_file", "{iso_commands}", "-commit"]

[tools.ecc]
argv = ["dvdisaster", "-i", "{iso_path}", "-mRS02", "-n", "{media_code}", "-c"]

[tools.burn]
argv = ["cdrskin", "dev={device}", "-sao", "--bdr_obs_exempt", "stream_recording=off",
        "speed={speed}", "-eject", "{iso_path}"]

[tools.probe]
argv = ["cdrskin", "dev={device}", "-minfo"]

[media_profiles.bdr-dl]
capacity_bytes    = 50050629632
max_payload_bytes = 41528898000    # capacity / 1.2052, 20% RS02
media_code        = "BD2"
speed             = "6"

[media_profiles.bdr-sl]
capacity_bytes    = 25025314816
max_payload_bytes = 20764449000
media_code        = "BD"
speed             = "6"

[media_profiles.dvd]
capacity_bytes    = 4700372992
max_payload_bytes = 3899744433
media_code        = "DVD"
speed             = "4"
```

Both ISO steps run native-mode `xorriso` over one command file bdar generates (`{iso_commands}`).
Format flags live in the template; only content lives in the file. Other ISO tools are out of
scope: mkisofs emulation can't `-alter_date`, so deduped paths would all carry one mtime. The
file's format is bdar's, not configurable:

```
-iso_rr_pattern off
-disk_pattern off
-map '<source path>' '/<on-disc path>'
-alter_date b @<mtime epoch> '/<on-disc path>' --
...
```

One `-map` and one `-alter_date` per live path; every word single-quoted with `'` written as
`'"'"'`. The pattern-off header is in the file so an override can't drop it (`-alter_date`
expands wildcards by default). `-print_size` is exact and `-rollback_end` stops `iso_size` from
writing the ISO at exit. `-hardlinks` is left out; extent sharing doesn't need it.

Templates:

- argv arrays executed directly. No shell. A pipeline is `["sh", "-c", "..."]` written out.
- Substitution is within-word (`dev={device}`); `{{` is a literal brace.
- Each template kind has its own closed placeholder set; a placeholder outside it, or a collision
  between a bdar name and a profile key, is a load error. Resolution happens at load, never
  mid-burn.
  - Pipeline steps: `{iso_path}` `{iso_commands}` `{volume_label}` `{device}` `{iso_sectors}`
    `{archive}` `{disc_number}` plus every key of the bound media profile.
  - `prepare`: `{root}` `{archive}`.
  - `freeze` / `release`: `{freeze_name}` `{archive}` `{archive_prefix}`.
  - `frozen_root`: `{freeze_name}`.
- Exit code is the contract. Non-zero fails the job. stdout/stderr go to the attempt log, never
  into the db. Only `iso_size` and `probe` output is parsed.
- Validation is strict on structure (malformed TOML, unknown step, unresolvable placeholder,
  empty required step, staging dir inside a root, a root outside the frozen prefix, a top-level
  name clashing with `README.txt` or `_ARCHIVE_INFO` case-insensitively, short name over 12
  chars, file device outside a test archive, frozen `short_name` / `test` mismatch) and advisory
  on content: the one known-destructive omission (`--bdr_obs_exempt` on `burn`) warns by
  substring presence.
- Drift: bdar carries a compiled-in list of the versions in which each step's default changed and
  warns when one is newer than an override's `forked_from`. An override with no `forked_from`
  warns once as "unknown base". No state is stored anywhere.

See ADR-0001 (open command templates) for why.

### Archive model

- An archive is N roots, one global exclude list, an `archive_prefix` and its own db.
- `archive_prefix` is the closest common parent of all roots, computed on the first `snapshot`
  and frozen in the db with `short_name` and `test`. A root outside it is a hard error. Every
  on-disc path is the source path relative to the prefix.
- `paths` stores only that one archive-relative path. There is no roots table.
- `prepare` runs on a root before the walk, before any freeze, and a non-zero exit aborts.
  A `prepare` staging directory belongs inside the prefix with a plain visible name.
- A `disc_extras` directory is copied into `_ARCHIVE_INFO\about-this-library\` on every disc.
  It is not archived content and is not pinned; later discs get the newer copy.

### Snapshot

`bdar snapshot [--dry-run] [--rehash]`:

1. Refuse if a disc is planned, staged or burned-but-unverified ("finish it or
   `burn --discard`").
2. Run each root's `prepare`.
3. If a freeze is configured, run `freeze` with `{freeze_name}` = `bdar-<snapshot_id>`, where
   the id is the one the commit will use (`MAX(id) + 1`; safe under the `flock`), and read
   everything below through `frozen_root`.
4. Walk the roots in on-disc path order, applying excludes. Symlinks are followed (layout from
   the link, content from the target, target may be anywhere; loops detected by visited inodes).
   Hardlinks are ordinary files. Zero-byte files are ordinary blobs. Empty directories are not
   recorded, and the README says so.
5. Hash: blake3 in-process, skipped when `(path, size, mtime)` matches the live `paths` row,
   unless `--rehash` or first run. Results go to a throwaway hash cache (a separate SQLite file in
   the XDG cache dir, one per archive) committed in batches, so an interrupted walk resumes.
6. Fail fast: unreachable root, failed `prepare`, unreadable file, symlink loop, dangling symlink
   each abort with nothing written to the archive db. A failed run releases its new freeze; the
   old one stays.
7. Commit atomically: new `snapshots` row, new `blobs` rows, open and close `paths` interval
   rows. An mtime-only change closes and reopens the row like any other change. A stealth edit
   (same size and mtime) is missed by design without `--rehash`. Then delete the hash cache and
   run `release` on the previous snapshot's freeze if it is still held.
8. Warn on file names differing only by case within one directory (Windows shows one of them)
   and on path components over 103 characters (Joliet truncates; xorriso keeps truncated names
   unique). Both are informational; nothing is stored.
9. Print a summary (added / changed / removed / bytes to burn) on stdout; progress on stderr.
   One line notes when no freeze is configured.

`--dry-run` walks and hashes (cache-assisted), prints the summary, commits nothing and takes no
freeze.

A moved or copied file closes its old `paths` row and opens a new one on the same blob. Nothing is
burned again, and the newest disc's `INDEX.csv` shows the new path on the original disc. There is
no rename detection beyond that, and no list of renames.

### Planning and packing

Done inside `burn`, lazily, one disc at a time, against the profile named on this run
(`--profile`, else the archive default). The profile and its capacity numbers are recorded on
the disc row.

- Remaining work = blobs referenced by a live path with no row in `disc_blobs` on a verified,
  non-retired disc.
- Unit: a blob with every live path pointing at it, never split. Ordered by first live path.
- Greedy strict path order, no gap fill: take the longest prefix whose estimate fits
  `max_payload_bytes - reserve - 1 MB`. Estimate = ~343 KB fixed + ~202 B/path
  + ~3.4 KB/new directory + sector-rounded data (fitted on the `prototype/disc-packing` branch).
- `reserve` = exact size of the fixed `_ARCHIVE_INFO\` parts (dvdisaster zips, `disc_extras`,
  README) + current db and index file sizes + a few MB of growth margin.
- Then the `iso_size` step on the full tree, `_ARCHIVE_INFO\` included. If it overshoots, drop
  paths from the tail, regenerate the db and index files, and check again. This loop converges
  whatever the template does.
- Oversize blob (bigger than this profile's `max_payload_bytes`): skipped, stays remaining work,
  reported as "complete except N oversize blobs for `<profile>`".
- Partial last disc (the planned disc is the last of the set and isn't full): ask; without a
  TTY stop unless `--allow-partial`.
- Disc numbers are global across the archive and fixed at plan time; the `discs` row and its
  `disc_blobs` rows are created then, because `{disc_number}` is in the volume label and the
  on-disc db copy must list the disc's contents. A discarded or re-planned disc's rows are
  deleted and its number freed.

### Staging

- One ISO in flight, in `staging_dir`. No pipelining.
- Space check before the disc row exists: free space ≥ profile `capacity_bytes` + 1 GiB.
- Build: write the xorriso command file (one `-map` per live path of each blob, all from one
  chosen source path per blob, the lexicographically first live path, so xorriso shares the
  extent; one `-alter_date` per path) and the per-disc `_ARCHIVE_INFO\` contents; run `mkiso`;
  run `ecc` (augments in place).
- One streaming pass after ECC: parse the ISO directory in-process (ISO9660 + Rock Ridge, no
  mount), then read the image once computing the full image blake3 and a blake3 per file extent
  compared to `blobs.hash`, and compare each path's Rock Ridge mtime to `paths.mtime`.
  `_ARCHIVE_INFO\` files have no blob and are covered by the image hash only. Doing this after
  ECC means damage done by the ECC tool can't be folded into the image hash.
- Any mismatch (extent hash or mtime) discards the ISO and stops: "staged ISO doesn't match the
  snapshot; take a new snapshot". No automatic rebuild, no prompt. With a freeze configured a
  mismatch means a bdar or template bug; without one the source changed, and a new snapshot
  re-plans around it. No media is spent either way.
- On success the disc is `staged` with image size and image hash recorded.
- Recovery: anything short of `staged` is rebuilt from source (no `dvdisaster -z`). A staged ISO
  not staged by this process is re-hashed against the recorded image hash before burning; missing
  or mismatched drops the disc to `planned`.
- The ISO is deleted only after `verified`.

### Burn loop and verification

`bdar burn [--count N] [--profile P] [--dry-run] [--discard] [--allow-partial] [--device D]`
loops until remaining work is empty or `--count` is reached. Each disc commits on its own.

Per disc:

1. Run `probe`; parse disc status (blank / written). Misreads are benign (cdrskin refuses a
   written disc; a blank fails the hash read). Only failing to tell blank from written is a
   hard failure.
2. Resume branch, by what is found:
   - planned with no ISO, or a partial ISO → rebuild;
   - staged with a recorded hash → reuse (re-hash if inherited);
   - burned-but-unverified and the disc in the drive is written and its first N bytes match the
     recorded image hash → verify (the ordinary post-burn read);
   - written and no match → `wrong_media` attempt, ask for a blank, never overwrite or append;
   - blank → burn.
3. Burn: the tool runs in its own process group. First SIGINT/SIGTERM during the write prints
   "burn in progress, disc will be a coaster — press Ctrl-C again within 10s to abort"; a second
   within the window kills the tool, records a `signal_abort` attempt, exits. SIGHUP is ignored
   during the write. An interrupted SAO write is always a coaster; resume is at pipeline level.
4. Reload: ask ("the reload clears the drive cache; leave the disc in"), then eject, close via
   ioctl, wait for ready; prompt "push the tray in" if the close fails. `auto_reload` or no TTY
   skips the question. A file device skips this.
5. Read: one routine shared by verify and scrub. `O_DIRECT`, first N = image size bytes (BD
   sessions round up to 32-sector clusters), one streaming blake3. Any read error ends the read
   as `unreadable`; a complete read with a different hash is `mismatch`.
6. Classify:
   - clean and matching → one transaction: `verified_at`, `last_read_at`, `last_read_result`.
     Then delete the ISO, eject.
   - `unreadable` → disc stays burned-but-unverified, no attempt row, "re-run `burn`". A second
     cold read that is also unreadable → ask "log as coaster and ask for a blank?"; automatic
     without a TTY.
   - `mismatch` → `hash_mismatch` attempt (coaster); ask for a blank; same disc number, same ISO.
7. Wait for the next blank: drive polling, Enter as a fallback with a TTY.

Every attempt writes a log file at
`$XDG_STATE_HOME/bdar/<archive>/logs/disc-<NNN>-attempt-<k>.log` (no config key) holding each
step's resolved argv, full output (progress `\r` lines thinned), exit code or signal, timings,
profile and the failure class. The db stores the file name, never the text.

Progress: one header line per step on stderr (`[disc 057] ecc: dvdisaster …`), then the tool's
own output live. In-process steps print bytes / rate / ETA. On failure the console prints failure
class, exit code, the last ~30 lines of the tool's stderr and the log path.

`burn --discard` deletes the staged ISO, the disc row and its `disc_blobs`, frees the number, and
records an `abandoned` attempt if a burn was tried. `burn --dry-run` plans in memory and prints
contents, size and estimated remaining disc count with no row, ISO or device access.

### Freeze lifecycle

bdar owns only the timing; it never runs `zfs` or `btrfs` itself.

- Order at snapshot: `prepare` → `freeze` (new) → walk → commit → `release` (old).
- `{freeze_name}` is `bdar-<snapshot_id>`; nothing else about the freeze is stored except
  `freeze_released_at`, so a release runs once. `release` runs when the set's remaining work is
  empty (oversize blobs included: they hold the freeze) or when the snapshot is superseded, on
  whichever host runs the triggering `burn` or `snapshot`. A failed release warns; the next
  `burn` or `snapshot` tries again because the column is still NULL. It never blocks work.
- One `frozen_root` stands in for `archive_prefix`; roots resolve under it because they're
  stored archive-relative. All roots must share one dataset or subvolume (or be mounted to look
  like it); documented limit.
- A missing frozen tree at `burn` is an ordinary unreadable-source error: fail fast, "take a new
  snapshot". Never a fallback to the live tree.
- Symlink targets are not frozen; accepted and documented.
- Defaults are empty; README carries ZFS and btrfs examples.

### Scrub, health, retirement

`bdar scrub` loops like `burn`: insert any disc, identify it from the volume label
`<SHORT_NAME>_<NNN>`, refuse other archives' discs, skip the reload cycle for a freshly inserted
disc, run the shared read routine over the full image size, write `last_read_at` and
`last_read_result`, eject, wait for the next. No sampling. `status` lists discs whose
`last_read_at` is older than `scrub_interval`, oldest first; retired discs are off the schedule.

Health is the latest read, never a separate state: a verified, unretired disc whose
`last_read_result` isn't `clean` is **degraded**. No read history and no trend.

Retirement is stored and final:

- `scrub` asks when a read comes back degraded ("Retire it? Its N blobs will be burned again");
  automatic without a TTY; "no" leaves it degraded and listed by `status`.
- `bdar retire <disc> --reason degraded|lost|unreadable [--note …] [--yes]`: verified discs
  only (planned/staged → "use `burn --discard`"); shows blobs, bytes and rough disc count
  returning to remaining work, then asks; `--yes` required without a TTY; no automatic path.
- A retired disc's copies stop counting. Its live blobs return to remaining work and the normal
  planner burns them onto new numbers with any new content. Deleted content isn't remade;
  renamed files are burned under current names; a blob with another verified non-retired copy
  isn't burned again. The retire message says "run `bdar snapshot`, then `bdar burn`".
- The old disc keeps its number; the user writes RETIRED on it; still-readable content is a
  spare copy. No un-retire.

### On-disc payload

```
<root>\
|-- README.txt
|-- _ARCHIVE_INFO\
|   |-- README.txt
|   |-- DISCS.txt           one line per disc: number, burn date, snapshot, first..last path;
|   |                       retired discs: "RETIRED <date> (<reason>): contents copied to later discs"
|   |-- FOLDERS.txt         every folder in the current tree -> the discs holding its files (~2 MB)
|   |-- INDEX.csv           path,disc,stored_as,size for every file in the current tree (no hash)
|   |-- archive.sqlite      the full db up to and including this disc
|   |-- about-this-library\ the archive's disc_extras, copied as-is
|   `-- tools\              dvdisaster: Windows GUI zip and Linux static CLI zip, kept zipped
`-- <archive-relative paths via graft points>
```

- All three index files describe the current tree only; history is in the db. They point each
  file at a non-retired verified copy, falling back to a retired copy only while the remake is
  in progress. `stored_as` is empty unless the file sits on disc under another name.
- Renames are never burned again: a file renamed or copied after it was burned stays on disc
  under its old name. The copy-every-disc restore yields all content in the older layout. No
  restore tool. Documented in the README; `stored_as` links the current name to the disc name.
- The db copy is made with `VACUUM INTO` into the staging dir, then `PRAGMA journal_mode=DELETE`
  on the copy, so no WAL sidecar is needed to open it. Indexes are kept. Read commands opening a
  db on read-only media use `immutable=1`. The copy shows its own disc as `planned` (it is written
  before `mkiso`); `DISCS.txt` and the README say the disc in hand was verified after its copy was
  written.
- Two dvdisaster zips go on disc, kept zipped: the Windows GUI build for a restorer and the Linux
  static CLI for a helper. The recovery workflow is "copy the zip off any disc that reads, then put
  in the bad one", and one file to drag plus Extract All is the easiest form. No bdar binary, no
  source tarball; nobody needs bdar to read the archive. A helper opens `archive.sqlite` with DB Browser for SQLite (named in the
  README) or `bdar --db`.
- `burn` copies both zips from `disc_tools_dir`. On a real (`test = false`) archive a missing
  zip is a hard error naming the expected path and the download URL.
- Volume label `<SHORT_NAME>_<NNN>`; Joliet caps at 16 characters, hence short name ≤ 12.
- Physical label: marker only — archive name, disc number, burn date. No "of N".
- README: plain text, CRLF, templated (archive name, disc number, date, size). Sections: what
  this is; getting everything back; getting one file or folder back; if a disc won't read (a
  walkthrough for a non-technical reader: copy the dvdisaster zip off a good disc, extract, read
  the bad disc to an image, fix it, double-click the `.iso`, copy files; the Linux CLI equivalent
  in a few lines; plus: a retired disc's
  files are on later discs, check the newest `INDEX.csv`); file names on Windows (103-character
  truncation, xorriso uniquifies collisions; empty folders aren't kept); the dvdisaster
  whole-device read (`-r --dao --ignore-iso-size`); the `find` SQL for someone with only a disc;
  the note that deduplicated files grow back to full size when copied; technical details,
  including how to continue the archive from this disc's db copy (copy the file off, then
  `burn --discard`; the physical disc stays a spare). Reference wording is on the
  `prototype/on-disc-payload` branch.
- Overhead about 0.4 GB per disc, reserved by the packer.

### CLI

Commands: `snapshot`, `burn`, `status`, `find`, `scrub`, `retire`, `config dump`. No `plan`,
`verify`, `init`, `abandon` or `import`. Global flags: `--config`, `--archive <name>` (optional
when one archive is defined), `--db <path>` (open that db read-only and immutable, for a disc's
copy; no config needed for `status` and `find`).

- `status [--json]`: archive name, prefix, db path; latest snapshot (date, superseded count);
  remaining work (blobs, bytes, estimated discs on the default profile); disc in flight and its
  state; last verified disc; overdue scrubs; degraded (unretired) discs; warnings for an
  unreleased freeze, oversize blobs and config drift.
- `find <pattern> [--at <date>] [--json]`: substring/glob on path → path, blob, disc(s), interval.
- `--json` on `status`, `find`, and the end-of-run summaries of `snapshot`, `burn`, `scrub`.
  Progress and logs always on stderr.
- Locking: `snapshot`, `burn`, `scrub`, `retire` take an exclusive `flock` on a lockfile beside
  the db for the whole run and refuse if held. `status` and `find` read without locking (SQLite
  WAL on the working db).
- No TTY: blank-wait polls only; partial last disc needs `--allow-partial`; reload is automatic;
  coasters and scrub retirements are automatic; `retire` needs `--yes`; the double-signal rule
  still applies.
- Cross-host: the db stores no hostnames; `device`, `frozen_root` and hooks live in each host's
  config. One host uses a db at a time; it moves by copying the file, never over a network
  filesystem.

### Schema

SQLite, one file per archive. Timestamps are unix seconds. Hashes are 32-byte blake3 BLOBs.
Derived state is never stored: set completeness, remaining work, superseded, disc state,
burned-but-unverified, health and scrub-overdue are queries.

```sql
-- One row, written by the first snapshot. Config disagreeing with it is a load error.
CREATE TABLE archive (
  id             INTEGER PRIMARY KEY CHECK (id = 1),
  archive_prefix TEXT    NOT NULL,
  short_name     TEXT    NOT NULL,
  test           INTEGER NOT NULL CHECK (test IN (0, 1)),
  schema_version INTEGER NOT NULL,
  created_at     INTEGER NOT NULL
);

-- A row exists only once the walk committed. The latest id is the live snapshot;
-- every other row is superseded. The freeze name is bdar-<id>.
CREATE TABLE snapshots (
  id                 INTEGER PRIMARY KEY,
  started_at         INTEGER NOT NULL,
  completed_at       INTEGER NOT NULL,
  freeze_released_at INTEGER            -- NULL while held, or when no freeze is configured
);

-- Content. Written at snapshot time; never updated.
CREATE TABLE blobs (
  hash BLOB    PRIMARY KEY,
  size INTEGER NOT NULL
) WITHOUT ROWID;

-- Tree position as interval rows. One row spans every snapshot where the path was
-- unchanged; a change of hash or mtime closes it and opens another. A live path has
-- removed_snapshot_id IS NULL. Renames are not detected. Directories have no rows.
CREATE TABLE paths (
  id                  INTEGER PRIMARY KEY,
  path                TEXT    NOT NULL,          -- archive-relative, forward slashes
  hash                BLOB    NOT NULL REFERENCES blobs (hash),
  mtime               INTEGER NOT NULL,
  added_snapshot_id   INTEGER NOT NULL REFERENCES snapshots (id),
  removed_snapshot_id INTEGER          REFERENCES snapshots (id),
  UNIQUE (path, added_snapshot_id)
);
CREATE INDEX paths_by_hash ON paths (hash);
CREATE INDEX paths_live    ON paths (path) WHERE removed_snapshot_id IS NULL;

-- One row per planned disc; global numbers fixed at plan time. State follows from the
-- timestamps: planned (none), staged (staged_at), verified (verified_at). A discarded
-- planned or staged disc is deleted.
CREATE TABLE discs (
  id                INTEGER PRIMARY KEY,
  disc_number       INTEGER NOT NULL UNIQUE,
  snapshot_id       INTEGER NOT NULL REFERENCES snapshots (id),
  planned_at        INTEGER NOT NULL,
  media_profile     TEXT    NOT NULL,            -- bound at plan time from the drive
  capacity_bytes    INTEGER NOT NULL,
  max_payload_bytes INTEGER NOT NULL,
  volume_label      TEXT    NOT NULL,
  image_size        INTEGER,                     -- bytes after ECC; the N of "first N bytes"
  image_hash        BLOB,                        -- blake3 of the final image
  staged_at         INTEGER,
  verified_at       INTEGER,                     -- the first clean read
  last_read_at      INTEGER,                     -- verify or scrub, whichever was latest
  last_read_result  TEXT CHECK (last_read_result IN ('clean', 'unreadable', 'mismatch')),
  retired_at        INTEGER,
  retired_reason    TEXT CHECK (retired_reason IN ('degraded', 'lost', 'unreadable')),
  retired_note      TEXT
);

-- Written at plan time; the on-disc db copy needs its own disc's contents. A row counts
-- as a copy only while its disc is verified and not retired.
CREATE TABLE disc_blobs (
  disc_id INTEGER NOT NULL REFERENCES discs (id),
  hash    BLOB    NOT NULL REFERENCES blobs (hash),
  PRIMARY KEY (disc_id, hash)
) WITHOUT ROWID;
CREATE INDEX disc_blobs_by_hash ON disc_blobs (hash);

-- One row per try at writing a disc; coasters are history. An unreadable verify adds no row.
CREATE TABLE burn_attempts (
  id             INTEGER PRIMARY KEY,
  disc_id        INTEGER NOT NULL REFERENCES discs (id),
  attempt_number INTEGER NOT NULL,
  started_at     INTEGER NOT NULL,
  ended_at       INTEGER,
  outcome        TEXT    NOT NULL CHECK (outcome IN
                   ('ok', 'signal_abort', 'tool_failed', 'hash_mismatch', 'wrong_media', 'abandoned')),
  media_profile  TEXT    NOT NULL,
  log_file       TEXT    NOT NULL,               -- file name only, never log text
  UNIQUE (disc_id, attempt_number)
);
```

Derivations, as the queries the code implements:

```sql
-- The live snapshot.
SELECT MAX(snapshots.id) FROM snapshots;

-- Remaining work: live blobs with no safe copy. The set is complete when this is empty.
-- Oversize blobs for the bound profile are filtered by size in code and still count.
SELECT blobs.hash, blobs.size
FROM blobs
WHERE EXISTS (
    SELECT 1 FROM paths
    WHERE paths.hash = blobs.hash AND paths.removed_snapshot_id IS NULL)
  AND NOT EXISTS (
    SELECT 1
    FROM disc_blobs
    INNER JOIN discs ON discs.id = disc_blobs.disc_id
    WHERE disc_blobs.hash = blobs.hash
      AND discs.verified_at IS NOT NULL
      AND discs.retired_at IS NULL);

-- Disc in flight: a planned, staged or burned-but-unverified disc (the guard for `snapshot`).
SELECT discs.id FROM discs WHERE discs.verified_at IS NULL;

-- Burned but unverified: the latest attempt wrote ok and no clean read followed.
SELECT discs.id
FROM discs
INNER JOIN burn_attempts ON burn_attempts.disc_id = discs.id
WHERE discs.verified_at IS NULL
  AND burn_attempts.attempt_number = (
    SELECT MAX(latest.attempt_number) FROM burn_attempts AS latest
    WHERE latest.disc_id = discs.id)
  AND burn_attempts.outcome = 'ok';

-- Degraded: verified, not retired, and the latest read wasn't clean.
SELECT discs.disc_number
FROM discs
WHERE discs.verified_at IS NOT NULL
  AND discs.retired_at IS NULL
  AND discs.last_read_result <> 'clean';

-- Scrub overdue, oldest first.
SELECT discs.disc_number, discs.last_read_at
FROM discs
WHERE discs.verified_at IS NOT NULL
  AND discs.retired_at IS NULL
  AND discs.last_read_at < unixepoch() - :scrub_interval_seconds
ORDER BY discs.last_read_at ASC;

-- Freezes to release: held, and either superseded or complete (remaining work empty).
SELECT snapshots.id
FROM snapshots
WHERE snapshots.freeze_released_at IS NULL
  AND (snapshots.id < (SELECT MAX(latest.id) FROM snapshots AS latest)
       OR NOT EXISTS (<remaining work>));
```

Hash cache (separate throwaway SQLite file, never in the archive db, never on disc): keyed by
`(path, size, mtime)` → `hash`, committed in batches during the walk, deleted on commit.

### Modules

Named by responsibility; file layout is the implementer's.

- **config**: TOML load, defaults overlay, profile merge, placeholder resolution, validation and
  drift warnings, `config dump`.
- **tools**: run a template (argv, process group, log capture), parse `iso_size` and `probe`.
- **archive db**: schema, migrations, the derivation queries, `flock`, `VACUUM INTO` copy.
- **snapshot**: prepare, freeze/release timing, walk, hash cache, atomic commit, case-collision
  and Joliet-length warnings.
- **planner**: remaining work, path-order packing, estimate, reserve, trim loop, oversize and
  partial-last-disc handling.
- **staging**: xorriso command file, `_ARCHIVE_INFO\` generation (README, DISCS/FOLDERS/INDEX, db
  copy, extras, dvdisaster zips), mkiso/ecc, the in-process ISO parser and streaming pass.
- **drive**: probe, eject/close/wait-ready, the shared read routine, file-device handling.
- **burn**: the loop, resume branches, signals, attempt rows, logs, verification transaction.
- **scrub / retire**: label identification, read, retirement.
- **cli**: commands, flags, TTY/no-TTY behaviour, `--json`, `--db`, `status`, `find`.
- **test support**: synthetic corpus (table-driven cases, sparse bulk generator), test media
  profile, rehearsal corpus example.

### Build and deployment

- AUR source package `bdar`, published manually per release. Depends on `libisoburn` (xorriso),
  `libburn` (cdrskin) and `dvdisaster-speed47` for host ECC. The PKGBUILD fetches the two
  dvdisaster zips (speed47 `v0.79.10-pl6`: `win64-portable` GUI, ~34 MB; `linux64-static-cli-only`,
  ~8 MB; by sha256) as `source=()` entries into `/usr/share/bdar/tools/`; bumping that pin is a
  one-line change.
- A non-AUR user downloads the zips once into `disc_tools_dir`; the `burn` error message names
  the path and URLs.
- GitHub Actions on push and PR: `cargo test` on Ubuntu with xorriso installed, plus clippy and
  fmt. `--ignored` tests never run in CI. No release build job.

## Testing Decisions

A good test exercises the binary or a command through its public seam and asserts on observable
outcomes: db rows, the disc state, what was reported, which files were read, what the ISO's
Joliet tree shows. Tests don't assert on internal structure.

Seams (settled; no new ones):

1. **The `bdar` binary with a test archive** (`test = true`, `device` is a regular file, burn
   template `["cp", "{iso_path}", "{device}"]`, shipped `probe` stub script printing
   cdrskin-style blank/written output by whether the file exists). The pipeline runs
   `planned → staged → verified` on the real code path. Covers packing, staging, the streaming
   pass, resume branches, `--discard`, retire-then-remake, signals (slow fake burn script: one
   SIGINT warns and continues; two abort with a `signal_abort` row and a cleaned-up process
   group; SIGHUP ignored), `mismatch` (edit a byte in the device file), `unreadable` (truncate
   the device file), and the reload prompt / `auto_reload` / no-TTY paths via the stub `probe`.
2. **Table-driven corpus cases**: a case is a sequence of library states with symbolic content
   tokens (`"M"` always yields the same bytes, size and mtime), each state optionally declaring
   `burn` (remaining work as tokens), `opened` / `closed` (interval rows), `reads` (files actually
   hashed), `rehash`, `burned` (which tokens get burned before the next state). Unset
   expectations aren't checked. Generated on demand into plain tempdirs; nothing binary committed;
   no ZFS. Cases: new, unchanged, modified in place, moved, copied, deleted, directory moved,
   always-identical pair, mtime-only change (closes and reopens, nothing to burn), stealth edit
   (missed by default, caught by `--rehash`; both asserted), unburned carryover, swapped contents,
   edit-then-revert, symlink followed, dangling symlink aborts with nothing written, unreadable
   file aborts, empty dir not recorded, zero-byte file, case collision warns.

Other decisions:

- Disc-boundary tests use a tiny test media profile (~20 MB `max_payload_bytes`) with a few
  hundred small real files, real hashing and real xorriso. The sparse bulk generator (photo-like
  size mix, duplicate rate, oversize files) serves packer tests that work from a listing. A
  ~200k-entry many-small-files disc is a smoke test for xorriso scale.
- Names: no Joliet parser in bdar. A default-set test builds an ISO from a long-name corpus
  (near-collisions sharing a 103-character prefix, non-ASCII names) and lists the Joliet tree
  with `xorriso -indev x.iso -find /` to assert what Windows sees.
- The on-disc db copy: a unit test opens a `VACUUM INTO` copy read-only with `immutable=1`.
- ECC: an opt-in test augments an image, overwrites scattered sectors, runs dvdisaster repair
  and checks the hash.
- Default `cargo test` needs only xorriso and runs in GitHub Actions. `#[ignore]` tests need
  dvdisaster and real cdrskin and fail loudly when a tool is missing.
- Real eject/close ioctls, the no-TTY reload and the physical scratch test are exercised only by
  the DVD rehearsal (`docs/rehearsal.md`: `REHEARSAL` archive on DVDs, on the burn host with real
  freeze hooks, seeded synthetic root plus optional real slice, 3 discs then a re-snapshot with
  renames and 1 more). Required checks: ceiling, burn loop, Windows read, restore from newest
  disc on Windows, marker scratch recovery with radial and concentric shapes, incremental renames
  not burned again, and a non-technical person repairing a scratched DVD on Windows using only the
  README and the GUI zip copied from another disc. That last check also settles whether the GUI's
  settings trim the RS02 ECC area (if so the README names the setting) and whether the GUI exe
  accepts CLI arguments. Disc 001 of the real archive is the BD-R gate.
- Prior art: none in the crate. The prototypes on the `prototype/*` branches are the reference
  for behaviour, not test code.

## Out of Scope

- Writing the implementation and performing the real burn (this is the spec; tickets follow).
- Binary deltas, compression, a GUI, move/rename detection, a restore tool, RS02 repair of a
  degraded disc, pipelining ISO builds, splitting oversize blobs, Joliet parsing in bdar, raw
  `SG_IO`, qpxtool scanning, the firmware cross-flash, media migration off BD-R, shared (network)
  db access, ZFS as a requirement, ISO tools other than xorriso.
- Cut by the scope review ([#22](https://github.com/rbuchberger/bdar/issues/22)): disc read
  history and per-cluster health data, a generic read-check step, drive identity records, a
  bdar binary on disc, a Windows bdar build, a dvdisaster source tarball on disc, stored
  case-collision confirmations, an
  `unobtainable` state for changed files, a fake test drive.
- Concrete library inventory: it stays in the gitignored `notes/` directory.

## Further Notes

- Rename table for reading older tickets: `index` command / indexing run → `snapshot`;
  re-index → take a new snapshot; `scan` step → `check` (since cut); "the index" (plain-text
  listing) → `INDEX.csv`.
- The hard requirement everything serves: the newest disc alone (db, `INDEX.csv`, README,
  dvdisaster) is enough to understand and restore the entire archive with no bdar installation
  and no other machine.
- Prototypes: `prototype/config-template-model` (config shape, template-vs-owned comparison),
  `prototype/disc-packing` (`pack.py`, overhead fit), `prototype/on-disc-payload` (README draft,
  layout, index samples), `prototype/synthetic-corpus` (`corpus.py`), `research/qpxtool-bh16ns40`.
