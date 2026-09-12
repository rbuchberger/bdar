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

**Root**:
A directory tree explicitly allowlisted into an archive. Archival originals go in roots; derived
output does not.
_Avoid_: source dir, include path

**Archive prefix**:
The closest common parent of all of an archive's roots, frozen in the database on the first run.
Every on-disc path is relative to it.

**Prepare**:
A command declared on a root and run before the indexer walks it, staging generated or rotating
content into place. Optional, and a no-op by default.

### Content

**Blob**:
A distinct file content, identified by its blake3 hash. A blob is content, never a location — the
same blob may appear at many paths.
_Avoid_: file, object

**Path**:
An archive-relative location where a blob appeared, recorded with the interval over which it was
there. History is a series of interval rows; renames are not detected.
_Avoid_: filename, entry

**Blob copy**:
The record that a particular blob is present on a particular disc. A blob may have copies on many
discs.

**Snapshot**:
The state of an archive as observed by one indexing run.

**Snapshot set**:
The discs that carry one snapshot's new blobs. A set may mix media types and may sit partially
burned indefinitely — that is a legitimate, resumable state, not an error.

### Media

**Disc**:
One piece of physical write-once media.
_Avoid_: disk (reserve that for magnetic storage)

**Media profile**:
A named description of a medium — its capacity, its payload ceiling, and any tool steps peculiar to
writing it. Built-ins ship for the common media; users may define their own. A disc's profile is
chosen when it is burned and recorded against it.

**Payload ceiling**:
The largest ISO that still fits a disc once ECC has been added. Declared by the media profile, so
that packing never has to reason about ECC overhead.

**ECC**:
Error-correction data added to an ISO after it is built, to survive later degradation. It lives
beyond the end of the ISO filesystem, so a mounted disc never exposes it.

**Scrub**:
A periodic read check of discs already burned, to catch degradation while copies can still be
remade.

### The pipeline

**Step**:
One named stage of turning a disc's worth of files into a burned disc by invoking an external tool.
The set of step names is fixed; a step with no command configured is skipped.

**Template**:
The argv array that configures a step. Executed directly, never through a shell.

**Placeholder**:
A `{name}` in a template, substituted before execution. Names come either from bdar or from the
media profile, and every one must resolve when the config loads.
