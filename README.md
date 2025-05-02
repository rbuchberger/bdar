# BDAR - Bare Disk Archiver

For when you want to put a bunch of files on a bunch of optical disks and not be the only person in
the world who might get them back off again.

---

WORK IN PROGRESS, not ready for use

---

## Features

- Split many files across many disks, in a sensible order, without doing it manually.
- Create incremental snapshots with new files, as well as lists of deleted and renamed files.
- Use hashes to detect renamed files, avoid saving an entirely new copy.
- Track down which snapshot and disk contains a particular file, without going through many of them.
- Use dvdisaster to add ECC data to generated ISOs, so maybe you'll actually be able to get the data
  off if there's a little degradation over the years.

## Non-features (things it doesn't do)

- Binary deltas
- GUI
- Compression
- Efficiently handle files that are large compared to the size of the media. This is made for saving
  25MiB RAW files; If you wanna save a bunch of 30GiB video files to 50GB blu-rays you're gonna
  waste a lot of space.

## Motivation

I wanted to back up a 2TiB photo library to optical media, with incremental updates as we add photos
and edit and reorganize existing ones. I wanted this backup to have maximum compatibility; if I get
hit by a bus I want my non-nerd wife to throw a disk at her windows pc and have no trouble reading
the contents.

### Why optical media?

It's append only. This means it's ransomware-proof and immune to backup misconfigurations. No, this
should not be your only backup and yes you can accomplish something similar for a lot less effort
with a rotation of USB HDDs, but I burned a lot of CDs in college and I'm kinda nostalgic for it so
I'm going with this.

### Why bare files on disk?

There are other solutions (the one I nearly went with is [dar](http://dar.linux.free.fr/))) that do
everything this project can do and more, better, but the disk you produce is weird and nobody will
know what to do with it unless you explain it to them. Sure I can plop a statically compiled binary
on every disk, but I'm still potentially inflicting the command line on my family and we all know
that's not cool.

## Dependencies

- sqlite
- nushell
- dvdisaster
- xorrisofs
