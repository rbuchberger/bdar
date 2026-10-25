# bdar

bdar archives a photo library to write-once optical media as a backup of last resort. Files are
written bare — no archive format — so that a non-technical person can put a disc in a Windows PC and
read them with no software and no command line. That constraint is the reason the project exists
instead of using `dar`, and it outranks efficiency everywhere the two conflict.

## Language

### The archive

**Archive**:
A named collection of roots backed up as one ongoing series of discs, with its own database. One
config file may describe several.
_Avoid_: repo, repository, backup set

**Test archive**:
An archive whose discs are not real media, used to exercise the whole pipeline without a drive. Its
blobs are never backed up, however far its discs get.
_Avoid_: dry run

**Rehearsal**:
A throwaway archive burned to cheap media to prove the whole pipeline on real hardware before the
real archive's first disc is burned. Unlike a test archive, its discs are real. Built to be rerun.
_Avoid_: dry run, test burn

**Root**:
A directory tree explicitly allowlisted into an archive. Archival originals go in roots; derived
output does not.
_Avoid_: source dir, include path

**Archive prefix**:
The closest common parent of all of an archive's roots, frozen in the database on the first run.
Every on-disc path is relative to it.

**Disc extras**:
A directory named in an archive's config whose contents are copied onto every disc, for notes
about the library itself. Unlike a root, it is not archived content; it rides along so the newest
disc explains itself.

**INDEX.csv**:
The plain-text listing on every disc of each file in the archive's current tree and the disc
holding it, readable with no software. The newest disc's copy is the one to trust.
_Avoid_: index, catalogue

**Prepare**:
A command declared on a root and run before a snapshot walks it, staging generated or rotating
content into place. Optional, and a no-op by default.

### Content

**Blob**:
A distinct file content, identified by its blake3 hash. A blob is content, never a location — the
same blob may appear at many paths.
_Avoid_: file, object

**Path**:
An archive-relative location where a blob appeared, recorded with the interval over which it was
there. History is a series of interval rows; renames are not detected, and a renamed path whose
blob is already safe is never burned again, so on disc it keeps its old name.
_Avoid_: filename, entry

**Blob copy**:
The record that a particular blob is present on a particular disc. A blob may have copies on many
discs.

**Snapshot**:
The state of an archive as observed by one walk of its roots. Taking a snapshot is the only thing
that reads the whole library; a new one supersedes the last.
_Avoid_: index, indexing run, scan

**Snapshot set**:
The discs that carry one snapshot's new blobs. A set may mix media types and may sit partially
burned indefinitely — that is a legitimate, resumable state, not an error. It is complete when every
blob its snapshot references has a copy on a verified disc.

**Remaining work**:
The blobs the latest snapshot references that have no copy on a verified disc yet. It is never
stored, only worked out from what is already recorded. When it is empty, the snapshot set is complete.

**Unobtainable blob**:
A blob the latest snapshot references but whose source changed or vanished after the snapshot was
taken, so it can no longer be read to put on a disc. It is skipped until a new snapshot picks up the
file's current content.

**Freeze**:
A read-only, point-in-time view of an archive's roots, taken by a user-configured command just
before a snapshot is taken and held until its snapshot set is complete or superseded. Taking the
snapshot and burning both read from it. Optional: without one, the live tree is read and anything that changed meanwhile is
caught before it reaches a disc.
_Avoid_: snapshot (that is the archive's observed state, not the filesystem's), ZFS snapshot

**Superseded snapshot**:
A snapshot that is no longer the latest. Its set is never completed; whatever it had left to burn
is carried by the newer snapshot's set. Only the latest snapshot is ever being burned.

### Media

**Disc**:
One piece of physical write-once media. Its number is global across the archive and is fixed when
the disc is planned, before it is burned.

**Verified disc**:
A disc whose burned image has been read back and matched. Until then, nothing on it counts as
archived.

**Burn attempt**:
One try at writing a disc's image to a blank. A failed attempt leaves a coaster and is kept as
history; the disc is retried on a fresh blank under the same number.
_Avoid_: disk (reserve that for magnetic storage)

**Case collision**:
Two names in one folder that differ only by letter case. Both are kept on disc, but Windows can
open only one of them, so burning them needs the user's explicit confirmation and the disc's README
lists them.

**Media profile**:
A named description of a medium — its capacity, its payload ceiling, and any tool steps peculiar to
writing it. Built-ins ship for the common media; users may define their own. A disc's profile is
chosen when it is burned and recorded against it.

**Payload ceiling**:
The largest ISO that still fits a disc once ECC has been added. Declared by the media profile, so
that packing never has to reason about ECC overhead.

**Oversize blob**:
A blob larger than the payload ceiling of the medium in the drive. It is never split; it waits as
remaining work until a disc with a larger ceiling is burned.

**ECC**:
Error-correction data added to an ISO after it is built, to survive later degradation. It lives
beyond the end of the ISO filesystem, so a mounted disc never exposes it.

**Scrub**:
A periodic read check of discs already burned, to catch degradation while copies can still be
remade.

**Degraded disc**:
A verified disc whose latest read found unreadable or mismatched data that persisted on a second
read. It needs remaking. A disc whose problem cleared on the re-read is only watched.

**Retired disc**:
A verified disc whose copies no longer count, because it degraded, was lost, or can't be read at
all. Its live blobs return to remaining work and are burned onto later discs under new numbers.
Retirement is final; the disc keeps its number, and anything still readable on it is a spare copy.
_Avoid_: remade disc (the disc itself is never remade, only its content)

### The pipeline

**Step**:
One named stage of turning a disc's worth of files into a burned disc by invoking an external tool.
The set of step names is fixed; a step with no command configured is skipped.

**Check**:
An optional step that reads a burned disc with an external tool, alongside verification and scrubs.
Its result is kept for a human to read and never counts as verification, which is bdar's own and
never a step.
_Avoid_: scan, verify

**Staging directory**:
Where the one disc image in flight is built and kept until its disc is verified. Never inside a
root.

**Template**:
The argv array that configures a step. Executed directly, never through a shell.

**Placeholder**:
A `{name}` in a template, substituted before execution. Names come either from bdar or from the
media profile, and every one must resolve when the config loads.
