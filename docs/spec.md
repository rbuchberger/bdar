# bdar design spec

The locked design for bdar, consolidated from the wayfinder map
([#1](https://github.com/rbuchberger/bdar/issues/1)) and its tickets, with every amendment applied
flat. Terms are those in `GLOSSARY.md`. Decisions here are settled; a ticket that contradicts one
must say so explicitly. Anything found contradictory or undecided while writing this is listed
under "Not yet specified" on the map, not decided here; those items are marked **open** below.

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
it, and puts a copy of that database, a plain-text index, a README and the tools onto every disc,
so the newest disc alone restores the whole archive.

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
14. As an archivist, I want symlinks followed, hardlinks treated as ordinary files, and empty
    directories and zero-byte files preserved, so that the disc reproduces what was there.
15. As an archivist, I want `snapshot --dry-run` to tell me what changed and how much needs
    burning, so that I can decide whether to start a set.
16. As an archivist, I want bdar to tell me about file names that differ only by case in one
    folder and ask before including them, so that I know Windows will only open one of them.
17. As an archivist, I want a warning for path components longer than Joliet allows, so that I
    know Windows will show a different name.
18. As an archivist, I want taking a new snapshot to supersede the old one and carry its unburned
    work forward, so that abandoning a set and folding in new photos are the same simple act.
19. As an archivist, I want `snapshot` to refuse while a disc is in flight, so that a disc never
    belongs to a superseded snapshot.

### Freezing the source

20. As an archivist with a snapshotting filesystem, I want to give bdar `freeze` and `release`
    commands and a frozen path, so that every disc of a set is built from one consistent tree.
21. As an archivist, I want bdar to generate the freeze name and record it, so that after a reboot
    nothing has to be searched for.
22. As an archivist, I want the freeze released only when the set is complete or superseded, so
    that it is never destroyed while a disc still needs it.
23. As an archivist, I want a failed release to warn and retry later, so that it never blocks a
    burn or undoes a verified disc.
24. As an archivist, I want `burn` to refuse, not fall back to the live tree, when the frozen tree
    is gone, so that a disc never silently contains different bytes than the snapshot recorded.
25. As a user without a snapshotting filesystem, I want bdar to work on the live tree with the
    same archive correctness, so that ZFS is a recommendation and not a requirement.

### Burning

26. As an archivist, I want `bdar burn` to plan the next disc, build it, burn it, verify it and
    move on to the next blank in a loop, so that burning sixty discs is sixty swaps and nothing
    else.
27. As an archivist, I want each disc planned only when a blank is in the drive, against that
    blank's media profile, so that one set may mix BD-R DL, BD-R SL and DVD.
28. As an archivist, I want discs packed in path order with no gap filling, so that each disc is
    one contiguous range of the tree and a lost disc costs a range, not a scatter.
29. As an archivist, I want a blob's paths never split across discs and identical files stored
    once per disc, so that duplicate copies in the library cost nothing extra.
30. As an archivist, I want each disc to land as close to the payload ceiling as possible on the
    first sizing call, so that media isn't wasted and planning isn't slow.
31. As an archivist, I want a file larger than the medium skipped and reported, never split and
    never a crash, so that one big video doesn't block the set.
32. As an archivist, I want to be asked before a partial last disc is burned, so that I can wait
    for more content instead of spending a blank on a quarter-full disc.
33. As an archivist, I want the staged ISO checked against the recorded hashes before any media
    is spent, so that a bad build or a file that changed mid-build is caught for free.
34. As an archivist, I want the burned disc read back and hashed, every sector including ECC, so
    that "verified" means the disc in my hand matches the image.
35. As an archivist, I want the drive cache defeated before the read-back, with bdar telling me
    to leave the disc in, so that verification reads the medium and not RAM.
36. As an archivist, I want a disc to count as archived only once it is verified, so that the
    database never describes a disc that doesn't exist.
37. As an archivist, I want a failed burn to keep the disc number and the staged ISO and retry on
    a fresh blank, so that a coaster costs one blank and nothing else.
38. As an archivist, I want every burn attempt kept as history with the drive, media ID and
    outcome, so that I can judge a bad media batch later.
39. As an archivist, I want `burn` to resume from whatever it finds (partial ISO, staged ISO,
    burned-but-unverified disc) with no separate recovery command, so that a crash or a
    week's gap is nothing special.
40. As an archivist, I want `burn` to probe the disc in the drive rather than assume, so that it
    never writes onto a disc that already has data.
41. As an archivist, I want a single Ctrl-C during a write to warn and a second within ten seconds
    to abort, so that a stray keypress doesn't make a coaster.
42. As an archivist, I want a dropped SSH session to leave the burn running, so that a network
    blip doesn't waste a disc.
43. As an archivist, I want a verification I/O error to be retryable by running `burn` again, and
    a clean read with the wrong hash to be a coaster, so that the two failures are handled
    differently.
44. As an archivist, I want a verification failure to say where on the disc it diverged, so that I
    can tell a layer-break problem from random damage.
45. As an archivist, I want a per-attempt log file with every command, its output, exit code,
    timings and drive facts, so that a coaster can be diagnosed.
46. As an archivist, I want `burn --discard` to drop an in-flight disc and free its number, so
    that I can get out of a bad state.
47. As an archivist, I want `burn --dry-run` to show what goes on the next disc, so that I can
    sanity-check a set before spending media.
48. As an archivist, I want a space check before anything is created, so that a build never
    fails forty gigabytes in.
49. As an archivist, I want a file that changed after the snapshot to be skipped as unobtainable
    with a pointer to re-snapshot, so that one edited photo doesn't stall the set.
50. As an archivist, I want the burn tool's own progress passed through live, so that I can see a
    forty-minute write is moving.

### Scrubbing and retiring

51. As an archivist, I want `bdar scrub` to read every sector of every disc I feed it and record
    the result, so that degradation shows as a trend before data is lost.
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
57. As an archivist, I want an optional ECC-aware read check (`check` step) recorded alongside
    bdar's own read, so that a tool like dvdisaster can add its opinion without being trusted
    for correctness.

### Reading the archive

58. As an archivist, I want `bdar find <pattern>` to tell me which disc holds a file, including as
    of a past date, so that one file comes back without touching sixty discs.
59. As an archivist, I want `bdar status` to summarise the archive: latest snapshot, remaining
    work, disc in flight, warnings, so that I know what to do next.
60. As an archivist, I want `--json` on read commands and run summaries, so that I can pipe
    them into my shell.

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
65. As a restorer, I want the README to tell me what to do when a disc won't read, so that a bad
    disc isn't the end.
66. As a helper, I want the full database and `INDEX.csv` on the newest disc, so that I can answer
    any question about the archive's history with only that disc.
67. As a helper, I want static `bdar` and `dvdisaster` binaries for Windows and Linux on every
    disc, so that `status`, `find` and ECC repair work on a machine that never had them.
68. As a helper, I want the README to show the dvdisaster command that reads the whole device,
    so that I don't trim the ECC off while trying to recover a disc.
69. As a helper, I want `DISCS.txt` to mark retired discs and say their content is on later
    discs, so that I don't chase a dead disc.

### Configuration and other media

70. As a user, I want bdar to run with no config file at all, so that the shipped defaults are the
    program and not an example to copy.
71. As a user, I want to override any tool's command line as an argv array, so that a different
    burner, a different medium or a different version of a tool needs no bdar release.
72. As a user, I want built-in media profiles for BD-R DL, BD-R SL and DVD and the ability to
    define my own with its own payload ceiling and tool overrides, so that an unforeseen medium is
    a config change.
73. As a user, I want `bdar config dump` to print the effective config and `--defaults` the
    shipped one, so that I can diff after an upgrade without a stale copy.
74. As a user, I want a warning when a shipped default I forked from has since changed, so that
    my override doesn't go stale silently.
75. As a user, I want a load-time error for a malformed template, unknown step or unresolvable
    placeholder, so that a config mistake fails before a burn, not during one.
76. As a user, I want an advisory warning when a template omits a known-destructive flag, so
    that I'm told without bdar pretending to understand another tool's arguments.
77. As a user, I want the correctness checks to be bdar's own code and not a template, so that I
    cannot weaken them by editing a string.
78. As a user, I want to disable ECC by emptying a template, so that "no ECC" needs no special
    case.

### Development and rehearsal

79. As a developer, I want a test archive whose device is a regular file, so that the whole
    pipeline runs to `verified` on the real code path with no drive.
80. As a developer, I want a file device refused outside a `test = true` archive, so that a
    misconfigured real archive can't report blobs as safe while they sit on an SSD.
81. As a developer, I want table-driven corpus tests written as sequences of library states, so
    that every dedup and deletion case has one definition.
82. As a developer, I want a fake drive that returns I/O errors on chosen ranges and fails tray
    control, so that the read-classification and reload logic is tested.
83. As a developer, I want the default `cargo test` to need only xorriso and run in CI, with
    tool-dependent tests opt-in and failing loudly when a tool is missing.
84. As an archivist, I want a written, rerunnable DVD rehearsal, so that the pipeline is proven on
    real hardware before the first expensive disc.
85. As an archivist, I want the first real disc to be a gate with a pause to inspect and time it,
    so that the bulk load is planned from a measurement.

## Implementation Decisions

### Codebase

- Keep the existing Rust crate and rewrite `src/` and the schema in place. Delete the nushell
  implementation. Keep the on-disc info material (dvdisaster binaries and manual).
- Dependencies already settled: `rusqlite` (bundled), `blake3` crate (in-process; never `b3sum`),
  `clap`, `walkdir`, TOML config. YAML is dropped.
- The crate must build static binaries for x86_64 Linux and x86_64 Windows; the burning host
  needs both available to put on discs. (Build/deploy pipeline is **open**.)

### Configuration

TOML at `$XDG_CONFIG_HOME/bdar/config.toml`, overridden by `--config`. Missing file means
compiled-in defaults. The file is a sparse overlay: only overrides are stated. `bdar config dump`
prints the effective config; `--defaults` prints the shipped one, with `forked_from` filled in so
copying a default pastes it.

Global keys:

- `device` — the drive, e.g. `/dev/sr0`; `--device` overrides. May be a regular file only in a
  test archive.
- `auto_reload` (default `false`) — skip the confirmation before the eject/reload cycle.
- `[tools.<step>]` — six fixed step names: `iso_size`, `mkiso`, `ecc`, `burn`, `check`,
  `probe`. Unknown name is a load error. Each has `argv` (array), optional `ack_warnings`
  (silence the advisory flag warnings for that step), optional `forked_from` (bdar version the
  override was copied from). Empty `argv` disables the step; `iso_size`, `mkiso`, `burn` and
  `probe` are required, so empty there is a load error. `iso_size` additionally has
  `stdout_is = "sectors"` (last integer on stdout). `probe`'s output contract is
  "cdrskin `-minfo` compatible", documented.
- `[media_profiles.<name>]` — `capacity_bytes`, `max_payload_bytes` (the packing ceiling; the
  packer uses only this and never reasons about ECC), and any other keys, exposed to templates as
  placeholders. A profile may contain `[media_profiles.<name>.tools.<step>]` overriding a step
  wholesale. Built-ins: `bdr-dl`, `bdr-sl`, `dvd`.
- `[archives.<name>]` — `roots` (**open**: list vs tables with names), `exclude` (global list of
  globs), `media_profile` (default profile), `db` (path; default location **open**),
  `staging_dir` (default `$XDG_CACHE_HOME/bdar/<archive>/`), `disc_extras` (directory path),
  `short_name` (≤ 12 chars, used in volume labels; whether frozen in the db is **open**),
  `test` (bool), `scrub_interval` (days, default 365), `frozen_root` (path template using
  `{freeze_name}`), and hook templates `freeze`, `release` (argv), plus `prepare` per root.

Shipped defaults (from the `prototype/config-template-model` branch; the `mkiso` mode is
**open**):

```toml
[tools.iso_size]
argv = ["xorrisofs", "-print-size", "-iso-level", "3", "-r", "-J", "-joliet-long",
        "-graft-points", "-path-list", "{path_list}"]
stdout_is = "sectors"

[tools.mkiso]
argv = ["xorrisofs", "-iso-level", "3", "-r", "-J", "-joliet-long", "-V", "{volume_label}",
        "-graft-points", "-path-list", "{path_list}", "-o", "{iso_path}"]

[tools.ecc]
argv = ["dvdisaster", "-i", "{iso_path}", "-mRS02", "-n", "{media_code}", "-c"]

[tools.burn]
argv = ["cdrskin", "dev={device}", "-sao", "--bdr_obs_exempt", "stream_recording=off",
        "speed={speed}", "-eject", "{iso_path}"]

[tools.check]
argv = ["dvdisaster", "-i", "{device}", "--dao", "--ignore-iso-size", "-v"]

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

Templates:

- argv arrays executed directly. No shell. A pipeline is `["sh", "-c", "..."]` written out.
- Substitution is within-word (`dev={device}`); `{{` is a literal brace.
- Each template kind has its own closed placeholder set; a placeholder outside it, or a collision
  between a bdar name and a profile key, is a load error. Resolution happens at load, never
  mid-burn.
  - Pipeline steps: `{iso_path}` `{path_list}` `{volume_label}` `{device}` `{iso_sectors}`
    `{archive}` `{disc_number}` plus every key of the bound media profile.
  - `prepare`: `{root}` `{archive}`.
  - `freeze` / `release`: `{freeze_name}` `{archive}` `{archive_prefix}`.
  - `frozen_root`: `{freeze_name}`.
- Exit code is the contract. Non-zero fails the job. stdout/stderr go to the attempt log, never
  into the db. Only `iso_size` and `probe` output is parsed.
- Validation is strict on structure (malformed TOML, unknown step, unresolvable placeholder,
  empty required step, staging dir inside a root, a root outside the frozen prefix, a top-level
  name clashing with `README.txt` or `_ARCHIVE_INFO` case-insensitively, short name over 12
  chars, file device outside a test archive) and advisory on content: the known-destructive
  omissions (`--bdr_obs_exempt` on `burn`, `--dao` on `check`) warn by substring presence.
- Drift: bdar carries a compiled-in list of the versions in which each step's default changed and
  warns when one is newer than an override's `forked_from`. An override with no `forked_from`
  warns once as "unknown base". No state is stored anywhere.

See ADR-0001 (open command templates) for why.

### Archive model

- An archive is N roots, one global exclude list, an `archive_prefix` and its own db.
- `archive_prefix` is the closest common parent of all roots, computed on the first `snapshot`
  and frozen in the db. A root outside it is a hard error. Every on-disc path is the source path
  relative to the prefix.
- `paths` stores only that one archive-relative path. There is no roots table.
- `prepare` runs on a root before the walk, before any freeze, and a non-zero exit aborts.
  A `prepare` staging directory belongs inside the prefix with a plain visible name.
- A `disc_extras` directory is copied into `_ARCHIVE_INFO\about-this-library\` on every disc.
  It is not archived content and is not pinned; later discs get the newer copy.

### Snapshot

`bdar snapshot [--dry-run] [--rehash] [--allow-case-collisions]`:

1. Retry any unreleased freeze from an earlier run.
2. Refuse if a disc is planned, staged or burned-but-unverified ("finish it or
   `burn --discard`").
3. Run each root's `prepare`.
4. If a freeze is configured, run `freeze` with a new `{freeze_name}` = `bdar-<archive>-<id>`
   and read everything below through `frozen_root`.
5. Walk the roots in on-disc path order, applying excludes. Symlinks are followed (layout from
   the link, content from the target, target may be anywhere; loops detected by visited inodes).
   Hardlinks are ordinary files. Empty directories and zero-byte files are recorded (representation
   of empty directories in the schema is **open**).
6. Hash: blake3 in-process, skipped when `(path, size, mtime)` matches the live `paths` row,
   unless `--rehash` or first run. Results go to a throwaway hash cache (a separate SQLite file in
   the XDG cache dir, one per archive) committed in batches, so an interrupted walk resumes.
7. Fail fast: unreachable root, failed `prepare`, unreadable file, symlink loop, dangling symlink
   each abort with nothing written to the archive db. A failed run releases its new freeze; the
   old one stays.
8. Commit atomically: new `snapshots` row, new `blobs` rows, open and close `paths` interval
   rows. An mtime-only change closes and reopens the row like any other change. A stealth edit
   (same size and mtime) is missed by design without `--rehash`. Then delete the hash cache and
   run `release` on the previous snapshot's freeze.
9. Report any new case collisions and require a typed `yes`; without a TTY refuse unless
   `--allow-case-collisions`. Confirmed pairs are stored and not asked again (table shape
   **open**). Warn on path components over 103 characters (informational; closing this is
   **open** on the map).
10. Print a summary (added / changed / removed / bytes to burn) on stdout; progress on stderr.
    One line notes when no freeze is configured.

`--dry-run` walks and hashes (cache-assisted), prints the summary, commits nothing and takes no
freeze.

### Planning and packing

Done inside `burn`, lazily, one disc at a time, against the profile named on this run
(`--profile`, else the archive default). The profile and its capacity numbers are recorded on
the disc row.

- Remaining work = blobs referenced by a live path with no copy on a verified, non-retired disc,
  minus the latest snapshot's unobtainable blobs.
- Unit: a blob with every live path pointing at it, never split. Ordered by first live path.
- Greedy strict path order, no gap fill: take the longest prefix whose estimate fits
  `max_payload_bytes - reserve - 1 MB`. Estimate = ~343 KB fixed + ~202 B/path
  + ~3.4 KB/new directory + sector-rounded data (fitted on the `prototype/disc-packing` branch).
- `reserve` = exact size of the fixed `_ARCHIVE_INFO\` parts (tools, `disc_extras`, README)
  + current db and index file sizes + a few MB of growth margin.
- Then the `iso_size` step on the full tree, `_ARCHIVE_INFO\` included. If it overshoots, drop
  paths from the tail, regenerate the db and index files, and check again. This loop converges
  whatever the template does.
- Oversize blob (bigger than this profile's `max_payload_bytes`): skipped, stays remaining work,
  reported as "complete except N oversize blobs for `<profile>`".
- Partial last disc (the planned disc is the last of the set and isn't full): ask; without a
  TTY stop unless `--allow-partial`.
- Disc numbers are global across the archive and fixed at plan time; the `discs` row is created
  then because `{disc_number}` is in the volume label. A discarded or re-planned disc's row is
  deleted and its number freed.
- Where an in-flight disc's planned blob list is persisted (needed by the on-disc db copy and by
  resume) is **open**.

### Staging

- One ISO in flight, in `staging_dir`. No pipelining.
- Space check before the disc row exists: free space ≥ profile `capacity_bytes` + 1 GiB.
- Build: write the path list (one graft point per live path of each blob; one chosen source
  path per blob, the lexicographically first live path, so xorriso shares the extent) and the
  per-disc `_ARCHIVE_INFO\` contents; run `mkiso`; run `ecc` (augments in place).
- One streaming pass after ECC: parse the ISO directory in-process (ISO9660 + Rock Ridge, no
  mount), then read the image once computing together the full image blake3, a blake3 per
  256 MiB chunk, and a blake3 per file extent compared to `blobs.hash`. `_ARCHIVE_INFO\` files
  have no blob and are covered by the image hash only. Doing this after ECC means damage done by
  the ECC tool can't be folded into the image hash.
- Pre-burn extent mismatch: re-hash the source. Source differs or is gone → record the blob as
  unobtainable, drop the disc to `planned`, re-plan without it, rebuild. Source still matches →
  the build corrupted it; hard stop. Freeze configured → any mismatch is a hard stop. More than
  ~1% of a disc's blobs unobtainable → stop and say re-snapshot.
- On success the disc is `staged` with image size, image hash and chunk hashes recorded.
- Recovery: anything short of `staged` is rebuilt from source (no `dvdisaster -z`). A staged ISO
  not staged by this process is re-hashed against the recorded image hash before burning; missing
  or mismatched drops the disc to `planned`.
- The ISO is deleted only after `verified`.

### Burn loop and verification

`bdar burn [--count N] [--profile P] [--dry-run] [--discard] [--allow-partial] [--device D]`
loops until remaining work is empty or `--count` is reached. Each disc commits on its own.

Per disc:

1. Run `probe`; parse disc status (blank / written) and media ID. Misreads are benign (cdrskin
   refuses a written disc; a blank fails the hash read). Only failing to tell blank from written
   is a hard failure; a media-ID parse failure records unknown.
2. Resume branch, by what is found:
   - planned with no ISO, or a partial ISO → rebuild;
   - staged with a recorded hash → reuse (re-hash if inherited);
   - burned-but-unverified and the disc in the drive is written and its first N bytes match the
     recorded image hash → verify (the ordinary post-burn read, `kind = probe_match`);
   - written and no match → `wrong_media` attempt, ask for a blank, never overwrite or append;
   - blank → burn.
3. Burn: the tool runs in its own process group. First SIGINT/SIGTERM during the write prints
   "burn in progress, disc will be a coaster — press Ctrl-C again within 10s to abort"; a second
   within the window kills the tool, records a `signal_abort` attempt, exits. SIGHUP is ignored
   during the write. An interrupted SAO write is always a coaster; resume is at pipeline level.
4. Reload: ask ("the reload clears the drive cache; leave the disc in"), then eject, close via
   ioctl, wait for ready; prompt "push the tray in" if the close fails. `auto_reload` or no TTY
   skips the question. A file device skips this and sysfs identity.
5. Read: one routine shared by verify, scrub and probe-match. `O_DIRECT`, first N = image size
   bytes (BD sessions round up to 32-sector clusters). On an I/O error drop to 64 KiB cluster
   reads for that chunk, record bad cluster LBAs, continue to the end. Compare chunk hashes to
   locate a mismatch.
6. Classify:
   - clean and matching → one transaction: `verified_at`, `disc_reads` row, `blob_copies` rows.
     Then the optional `check` step (exit code and log file recorded, never parsed, never
     blocking), delete the ISO, eject.
   - I/O error (bad clusters) → disc stays burned-but-unverified, no attempt row, "re-run `burn`".
     A second cold read with bad clusters → ask "log as coaster and ask for a blank?"; automatic
     without a TTY.
   - clean read, wrong hash → `hash_mismatch` attempt (coaster) with the first bad chunk; ask
     for a blank; same disc number, same ISO.
7. Wait for the next blank: drive polling, Enter as a fallback with a TTY.

Every attempt writes a log file (location **open**: `<db dir>/logs/` vs XDG state dir) named
`disc-<NNN>-attempt-<k>.log` holding each step's resolved argv, full output (progress `\r` lines
thinned), exit code or signal, timings, profile, drive vendor/model/firmware (sysfs), media ID and
the failure class. The db stores the file name, never the text.

Progress: one header line per step on stderr (`[disc 057] ecc: dvdisaster …`), then the tool's
own output live. In-process steps print bytes / rate / ETA. On failure the console prints failure
class, exit code, the last ~30 lines of the tool's stderr, mismatch location and log path.

`burn --discard` deletes the staged ISO and the disc row, frees the number, and records an
`abandoned` attempt if a burn was tried. `burn --dry-run` plans in memory and prints contents,
size and estimated remaining disc count with no row, ISO or device access.

### Freeze lifecycle

- Order at snapshot: `prepare` → `freeze` (new) → walk → commit → `release` (old).
- `release` runs when the set's remaining work is empty (including oversize blobs: they hold the
  freeze) or when the snapshot is superseded, on whichever host runs the triggering `burn` or
  `snapshot`. A failed release warns, is marked unreleased (representation **open**), and is
  retried at the start of the next `burn` or `snapshot`. It never blocks work.
- `burn` with a missing frozen tree refuses: "frozen tree missing, re-snapshot".
- Symlink targets are not frozen; accepted and documented.
- Defaults are empty; README carries ZFS and btrfs examples. `zfs hold` is mentioned, not default.

### Scrub, health, retirement

`bdar scrub` loops like `burn`: insert any disc, identify it from the volume label
`<SHORT_NAME>_<NNN>`, refuse other archives' discs, skip the reload cycle for a freshly inserted
disc, run the shared read routine over the full image size, record a `disc_reads` row
(`kind = scrub`), run `check`, eject, wait for the next. No sampling. `status` lists discs whose
latest clean read is older than `scrub_interval`, oldest first; retired discs are off the schedule.

Health is derived from the latest `disc_reads` rows, never stored:

- **degraded**: a bad cluster or chunk mismatch that persisted on a second cold read;
- **watch**: non-zero `check` exit, or a bad cluster that cleared on re-read;
- read-speed trend: informational, compared within one drive model, never a state.

Retirement is stored and final:

- `scrub` asks when it finds a disc degraded ("Retire it? Its N blobs will be burned again");
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
|   |-- archive.sqlite      the full db up to and including this disc (WAL question **open**)
|   |-- about-this-library\ the archive's disc_extras, copied as-is
|   `-- tools\              static bdar + dvdisaster for Windows and Linux; dvdisaster manual
`-- <archive-relative paths via graft points>
```

- All three index files describe the current tree only; history is in the db. They point each
  file at a non-retired verified copy, falling back to a retired copy only while the remake is
  in progress. `stored_as` is empty unless the file sits on disc under another name.
- Renames are never burned again: a file renamed or copied after it was burned stays on disc
  under its old name. The copy-every-disc restore yields all content in the older layout. No
  restore tool. Documented in the README; `stored_as` links the current name to the disc name.
- The db copy's own disc entry cannot be `verified` (it is written before the burn); exact
  wording is **open**.
- Volume label `<SHORT_NAME>_<NNN>`; Joliet caps at 16 characters, hence short name ≤ 12.
- Physical label: marker only — archive name, disc number, burn date. No "of N".
- README: plain text, CRLF, templated (archive name, disc number, date, size). Sections: what
  this is; getting everything back; getting one file or folder back; if a disc won't read
  (including: a retired disc's files are on later discs, check the newest INDEX.csv); file
  names on Windows (103-character truncation, xorriso uniquifies collisions); files Windows
  can't tell apart (only on discs carrying case collisions, with the copy-rename-copy
  workaround); the dvdisaster whole-device read (`-r --dao --ignore-iso-size`); the `find` SQL
  for someone with only a disc; the note that deduplicated files grow back to full size when
  copied; technical details. Reference wording is on the `prototype/on-disc-payload` branch.
- Overhead about 0.4 GB per disc, reserved by the packer.

### CLI

Commands: `snapshot`, `burn`, `status`, `find`, `scrub`, `retire`, `config dump`. No `plan`,
`verify`, `init` or `abandon`. Global flags: `--config`, `--archive <name>` (optional when one
archive is defined).

- `status [--json]`: archive name, prefix, db path; latest snapshot (date, superseded count);
  remaining work (blobs, bytes, estimated discs on the default profile); disc in flight and its
  state; last verified disc; overdue scrubs; degraded (unretired) and watch discs; warnings for
  unreleased freezes, oversize blobs, unobtainable blobs, case collisions in the set, config
  drift.
- `find <pattern> [--at <date>] [--json]`: substring/glob on path → path, blob, disc(s), interval.
- `--json` on `status`, `find`, and the end-of-run summaries of `snapshot`, `burn`, `scrub`.
  Progress and logs always on stderr.
- Locking: `snapshot`, `burn`, `scrub`, `retire` take an exclusive `flock` on a lockfile beside
  the db for the whole run and refuse if held. `status` and `find` read without locking (SQLite
  WAL on the working db).
- No TTY: blank-wait polls only; partial last disc needs `--allow-partial`; reload is automatic;
  coasters and scrub retirements are automatic; `retire` needs `--yes`; case collisions need
  `--allow-case-collisions`; the double-signal rule still applies.
- Cross-host: the db stores no hostnames; `device`, `frozen_root` and hooks live in each host's
  config. One host uses a db at a time; it moves by copying the file, never over a network
  filesystem.

### Schema

SQLite, one file per archive. Timestamps are unix seconds. Hashes are 32-byte blake3 BLOBs.
Derived state is never stored: set completeness, remaining work, superseded, disc health and
scrub-overdue are queries. Items marked open are on the map, not decided here.

```sql
-- One row, written by the first snapshot.
CREATE TABLE archive (
  id             INTEGER PRIMARY KEY CHECK (id = 1),
  archive_prefix TEXT    NOT NULL,
  schema_version INTEGER NOT NULL,
  created_at     INTEGER NOT NULL
  -- open: short_name, test
);

-- A row exists only once the walk committed. The latest id is the live snapshot;
-- every other row is superseded.
CREATE TABLE snapshots (
  id                 INTEGER PRIMARY KEY,
  started_at         INTEGER NOT NULL,
  completed_at       INTEGER NOT NULL,
  freeze_name        TEXT,              -- NULL when no freeze is configured
  frozen_root        TEXT,              -- resolved frozen_root for this snapshot
  freeze_released_at INTEGER            -- open: how a failed release is marked
);

-- Content. Written at snapshot time; never updated.
CREATE TABLE blobs (
  hash BLOB    PRIMARY KEY,
  size INTEGER NOT NULL
) WITHOUT ROWID;

-- Tree position as interval rows. One row spans every snapshot where the path was
-- unchanged; a change of hash or mtime closes it and opens another. A live path has
-- removed_snapshot_id IS NULL. Renames are not detected.
-- open: how empty directories are represented.
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

-- One row per planned disc; global numbers fixed at plan time. A discarded planned
-- or staged disc is deleted. State (planned / staged / verified) follows from
-- staged_at and verified_at; whether a state column exists as well is open.
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
  chunk_hashes      BLOB,                        -- packed 32-byte blake3 per 256 MiB chunk
  staged_at         INTEGER,
  verified_at       INTEGER,                     -- first clean verify read
  retired_at        INTEGER,
  retired_reason    TEXT CHECK (retired_reason IN ('degraded', 'lost', 'unreadable')),
  retired_note      TEXT,
  retired_by_read   INTEGER REFERENCES disc_reads (id)  -- NULL for a lost disc
);
-- open: where an in-flight disc's planned blob list lives before verification.

-- Inserted only in the transaction that verifies a disc. A blob is safe when it has a
-- copy on a verified, non-retired disc.
CREATE TABLE blob_copies (
  hash    BLOB    NOT NULL REFERENCES blobs (hash),
  disc_id INTEGER NOT NULL REFERENCES discs (id),
  PRIMARY KEY (hash, disc_id)
) WITHOUT ROWID;

-- One row per try at writing a disc; coasters are history. A verify I/O error adds no row.
CREATE TABLE burn_attempts (
  id              INTEGER PRIMARY KEY,
  disc_id         INTEGER NOT NULL REFERENCES discs (id),
  attempt_number  INTEGER NOT NULL,
  started_at      INTEGER NOT NULL,
  ended_at        INTEGER,
  outcome         TEXT    NOT NULL CHECK (outcome IN
                    ('ok', 'signal_abort', 'tool_failed', 'hash_mismatch', 'wrong_media', 'abandoned')),
  media_profile   TEXT    NOT NULL,
  drive_vendor    TEXT,                          -- sysfs; 'file' for a file device
  drive_model     TEXT,
  drive_firmware  TEXT,
  media_id        TEXT,                          -- from probe; NULL when unparsed
  write_speed     TEXT,                          -- as requested in the template
  first_bad_chunk INTEGER,                       -- hash_mismatch only
  log_file        TEXT    NOT NULL,              -- file name only, never log text
  UNIQUE (disc_id, attempt_number)
);

-- One row per full read of a burned disc. "Verified on <date>" is the first clean
-- verify read; health is derived from the latest rows.
CREATE TABLE disc_reads (
  id                INTEGER PRIMARY KEY,
  disc_id           INTEGER NOT NULL REFERENCES discs (id),
  burn_attempt_id   INTEGER          REFERENCES burn_attempts (id),  -- verify reads
  kind              TEXT    NOT NULL CHECK (kind IN ('verify', 'scrub', 'probe_match')),
  started_at        INTEGER NOT NULL,
  duration_seconds  INTEGER NOT NULL,
  drive_vendor      TEXT,
  drive_model       TEXT,
  drive_firmware    TEXT,
  result            TEXT    NOT NULL CHECK (result IN ('clean', 'unreadable', 'mismatch')),
  bad_cluster_count INTEGER NOT NULL DEFAULT 0,
  bad_clusters      BLOB,                        -- packed LBAs of failing 64 KiB clusters
  mismatched_chunks BLOB,                        -- packed chunk indices
  chunk_read_times  BLOB,                        -- packed per-chunk read time
  check_exit_code   INTEGER,                     -- NULL when the check step is disabled
  check_log_file    TEXT
);
CREATE INDEX disc_reads_by_disc ON disc_reads (disc_id, started_at);

-- Per snapshot, so a new snapshot clears it implicitly.
CREATE TABLE unobtainable_blobs (
  snapshot_id INTEGER NOT NULL REFERENCES snapshots (id),
  hash        BLOB    NOT NULL REFERENCES blobs (hash),
  detected_at INTEGER NOT NULL,
  reason      TEXT    NOT NULL CHECK (reason IN ('changed', 'missing')),
  PRIMARY KEY (snapshot_id, hash)
) WITHOUT ROWID;

-- open: case-collision confirmations (per snapshot vs per archive; columns).
```

Derivations, as the queries the code implements:

```sql
-- The live snapshot.
SELECT MAX(snapshots.id) FROM snapshots;

-- Remaining work: live blobs with no safe copy, minus the live snapshot's unobtainable blobs.
SELECT blobs.hash, blobs.size
FROM blobs
WHERE EXISTS (
    SELECT 1 FROM paths
    WHERE paths.hash = blobs.hash AND paths.removed_snapshot_id IS NULL)
  AND NOT EXISTS (
    SELECT 1
    FROM blob_copies
    INNER JOIN discs ON discs.id = blob_copies.disc_id
    WHERE blob_copies.hash = blobs.hash
      AND discs.verified_at IS NOT NULL
      AND discs.retired_at IS NULL)
  AND NOT EXISTS (
    SELECT 1 FROM unobtainable_blobs
    WHERE unobtainable_blobs.hash = blobs.hash
      AND unobtainable_blobs.snapshot_id = (SELECT MAX(snapshots.id) FROM snapshots));
-- The set is complete when this is empty. Oversize blobs for the bound profile are
-- filtered by size in code, and still count as remaining work.

-- Disc in flight: a planned or staged disc (the guard for `snapshot`).
SELECT discs.id FROM discs WHERE discs.verified_at IS NULL;

-- Scrub overdue, oldest first.
SELECT discs.disc_number, MAX(disc_reads.started_at) AS last_clean_read
FROM discs
INNER JOIN disc_reads ON disc_reads.disc_id = discs.id AND disc_reads.result = 'clean'
WHERE discs.verified_at IS NOT NULL AND discs.retired_at IS NULL
GROUP BY discs.id
HAVING last_clean_read < unixepoch() - :scrub_interval_seconds
ORDER BY last_clean_read ASC;
```

Hash cache (separate throwaway SQLite file, never in the archive db, never on disc): keyed by
`(path, size, mtime)` → `hash`, committed in batches during the walk, deleted on commit.

### Modules

Named by responsibility; file layout is the implementer's.

- **config**: TOML load, defaults overlay, profile merge, placeholder resolution, validation and
  drift warnings, `config dump`.
- **tools**: run a template (argv, process group, log capture), parse `iso_size` and `probe`.
- **archive db**: schema, migrations, the derivation queries, `flock`.
- **snapshot**: prepare, freeze/release timing, walk, hash cache, atomic commit, case
  collisions, Joliet-length warning.
- **planner**: remaining work, path-order packing, estimate, reserve, trim loop, oversize and
  partial-last-disc handling.
- **staging**: path list, `_ARCHIVE_INFO\` generation (README, DISCS/FOLDERS/INDEX, db copy,
  extras, tools), mkiso/ecc, the in-process ISO parser and streaming pass, unobtainable re-plan.
- **drive**: probe, eject/close/wait-ready, sysfs identity, the shared read routine with cluster
  fallback, the fake-drive seam (test only), file-device handling.
- **burn**: the loop, resume branches, signals, attempt rows, logs, verification transaction.
- **scrub / retire**: label identification, read, health derivation, retirement.
- **cli**: commands, flags, TTY/no-TTY behaviour, `--json`, `status`, `find`.
- **test support**: synthetic corpus (table-driven cases, sparse bulk generator), test media
  profile, rehearsal corpus example.

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
   group; SIGHUP ignored), and mismatches (edit a byte in the device file).
2. **Table-driven corpus cases**: a case is a sequence of library states with symbolic content
   tokens (`"M"` always yields the same bytes, size and mtime), each state optionally declaring
   `burn` (remaining work as tokens), `opened` / `closed` (interval rows), `reads` (files actually
   hashed), `rehash`, `burned` (which tokens get burned before the next state). Unset
   expectations aren't checked. Generated on demand into plain tempdirs; nothing binary committed;
   no ZFS. Cases: new, unchanged, modified in place, moved, copied, deleted, directory moved,
   always-identical pair, mtime-only change (closes and reopens, nothing to burn), stealth edit
   (missed by default, caught by `--rehash`; both asserted), unburned carryover, swapped contents,
   edit-then-revert, symlink followed, dangling symlink aborts with nothing written, unreadable
   file aborts, empty dir preserved, zero-byte file, case-collision confirm/refuse.
3. **One fake drive abstraction** (read + eject/close/wait-ready), test code only. Returns
   `EIO` on chosen LBA ranges and fails tray control. Tests the read classification (bad cluster
   → cluster reads, LBAs recorded, `unreadable`, retryable, second cold read decides coaster or
   degraded; mismatch → `hash_mismatch` with chunk located), degraded vs watch at scrub, and the
   reload logic (prompt, no-TTY auto, `auto_reload`, "push the tray in").

Other decisions:

- Disc-boundary tests use a tiny test media profile (~20 MB `max_payload_bytes`) with a few
  hundred small real files, real hashing and real xorriso. The sparse bulk generator (photo-like
  size mix, duplicate rate, oversize files) serves packer tests that work from a listing. A
  ~200k-entry many-small-files disc is a smoke test for xorriso scale.
- Names: no Joliet parser in bdar. A default-set test builds an ISO from a long-name corpus
  (near-collisions sharing a 103-character prefix, non-ASCII names) and lists the Joliet tree
  with `xorriso -indev x.iso -find /` to assert what Windows sees.
- ECC: an opt-in test augments an image, overwrites scattered sectors, runs dvdisaster repair
  and checks the hash.
- Default `cargo test` needs only xorriso and runs in GitHub Actions. `#[ignore]` tests need
  dvdisaster and real cdrskin and fail loudly when a tool is missing.
- Real eject/close ioctls, the no-TTY reload and the physical scratch test are exercised only by
  the DVD rehearsal (`docs/rehearsal.md`: `REHEARSAL` archive on DVDs, on the burn host with real
  freeze hooks, seeded synthetic root plus optional real slice, 3 discs then a re-snapshot with
  renames and 1 more; six required checks: ceiling, burn loop, Windows read, restore from newest
  disc on Windows, marker scratch recovery with radial and concentric shapes, incremental
  renames not burned again). Disc 001 of the real archive is the BD-R gate.
- Prior art: none in the crate. The prototypes on the `prototype/*` branches are the reference
  for behaviour, not test code.

## Out of Scope

- Writing the implementation and performing the real burn (this is the spec; tickets follow).
- Binary deltas, compression, a GUI, move/rename detection, a restore tool, RS02 repair of a
  degraded disc, pipelining ISO builds, splitting oversize blobs, Joliet parsing in bdar, raw
  `SG_IO`, qpxtool scanning as a required step, the firmware cross-flash, media migration off
  BD-R, shared (network) db access, ZFS as a requirement.
- Concrete library inventory: it stays in the gitignored `notes/` directory.

## Further Notes

- Rename table for reading older tickets: `index` command / indexing run → `snapshot`;
  re-index → take a new snapshot; `scan` step → `check`; "the index" (plain-text listing) →
  `INDEX.csv`.
- The hard requirement everything serves: the newest disc alone (db, `INDEX.csv`, README,
  tools) is enough to understand and restore the entire archive with no bdar installation and
  no other machine.
- Prototypes: `prototype/config-template-model` (config shape, template-vs-owned comparison),
  `prototype/disc-packing` (`pack.py`, overhead fit), `prototype/on-disc-payload` (README draft,
  layout, index samples), `prototype/synthetic-corpus` (`corpus.py`), `research/qpxtool-bh16ns40`.
- Open items (all on the map under "Not yet specified"): `mkiso` default mode; roots config
  shape; log file location; case-collision storage; short name and `test` frozen or not; on-disc
  db and WAL; db-copy wording for its own disc; disc state column vs derived and unreleased-freeze
  marking; Joliet warning severity; `-hardlinks` in defaults; build and deployment; empty
  directories in `paths`; where an in-flight disc's plan is persisted; default db location.
