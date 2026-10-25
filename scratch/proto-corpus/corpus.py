#!/usr/bin/env python3
"""
PROTOTYPE -- throwaway. Answers "Synthetic corpus for index and dedup tests".

Question: what does a synthetic corpus generator produce, and how does a test
say what it expects?

Two modes:

  corpus.py cases [-v] [NAME ...]
      Each case is a sequence of library states. Every state is written to a
      real directory, run through a toy indexer (same rules as the settled
      design: size+mtime skips re-hashing, interval rows for paths, a blob is
      safe once burned), and checked against the case's declared expectation.

  corpus.py bulk OUT --files N [--dup-rate R] [--seed S] [--oversize K]
      Writes a sparse tree with a photo-library size distribution for packer
      and disc-boundary tests. Each file has a unique header so every file is
      a distinct blob, but the tail is a hole, so disk use stays tiny.

Content is written as symbolic tokens: "X" always produces the same bytes,
size and mtime, so a test reads like algebra -- {a: X} -> {b: X} is a move.
blake2b stands in for blake3 (stdlib only).
"""
import argparse, hashlib, os, random, shutil, sys, tempfile

TOKENS = {}  # hash -> token, so results print symbolically


def _h(s):
    return hashlib.blake2b(s.encode(), digest_size=8).digest()


class F:
    """A file in a state: a content token, with optional size/mtime overrides."""

    def __init__(self, token, size=None, mtime=None):
        self.token = token
        self.size = size if size is not None else 1000 + int.from_bytes(_h("s" + token)) % 9000
        self.mtime = mtime if mtime is not None else 1_600_000_000 + int.from_bytes(_h("m" + token)) % 10**7


def as_file(v):
    return v if isinstance(v, F) else F(v)


def content(token, size):
    seed = _h("c" + token)
    return (seed * (size // len(seed) + 1))[:size]


def write_tree(root, tree):
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(root)
    for path, v in tree.items():
        f = as_file(v)
        full = os.path.join(root, path)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        data = content(f.token, f.size)
        with open(full, "wb") as fh:
            fh.write(data)
        os.utime(full, (f.mtime, f.mtime))
        TOKENS[hashlib.blake2b(data).digest()] = f.token


# ---- toy indexer: the rules settled in Schema / Snapshot sets -------------

class Archive:
    def __init__(self):
        self.current = {}  # path -> (hash, size, mtime): open interval rows
        self.safe = set()  # hashes with a copy on a verified disc

    def index(self, root, rehash=False):
        reads, seen = 0, {}
        for d, _, files in os.walk(root):
            for name in files:
                full = os.path.join(d, name)
                path = os.path.relpath(full, root)
                st = os.stat(full)
                prev = self.current.get(path)
                if prev and not rehash and prev[1:] == (st.st_size, int(st.st_mtime)):
                    seen[path] = prev  # trusted: no read
                    continue
                with open(full, "rb") as fh:
                    h = hashlib.blake2b(fh.read()).digest()
                reads += 1
                seen[path] = (h, st.st_size, int(st.st_mtime))
        opened = {p for p, row in seen.items() if self.current.get(p) != row}
        closed = {p for p, row in self.current.items() if seen.get(p) != row}
        self.current = seen
        remaining = {row[0] for row in seen.values()} - self.safe
        return dict(reads=reads, opened=opened, closed=closed, burn=remaining)

    def burn(self, hashes):
        self.safe |= hashes


# ---- cases ------------------------------------------------------------------

class Step:
    """One library state and what indexing it should produce.

    burn: tokens left to burn after indexing (the remaining work).
    burned: tokens actually burned before the next step (default: all of burn).
    Any expectation left as None is not checked.
    """

    def __init__(self, tree, burn=None, opened=None, closed=None, reads=None,
                 rehash=False, burned=None):
        self.tree, self.rehash, self.burned = tree, rehash, burned
        self.expect = dict(burn=burn, opened=opened, closed=closed, reads=reads)


S = Step
CASES = {
    "new file": [
        S({}),
        S({"a.jpg": "N"}, burn={"N"}, opened={"a.jpg"}, closed=set(), reads=1),
    ],
    "unchanged file": [
        S({"a.jpg": "U"}),
        S({"a.jpg": "U"}, burn=set(), opened=set(), closed=set(), reads=0),
    ],
    "modified at same path": [
        S({"a.jpg": "X"}),
        S({"a.jpg": "X2"}, burn={"X2"}, opened={"a.jpg"}, closed={"a.jpg"}, reads=1),
    ],
    "moved file": [
        S({"x/a.jpg": "M"}),
        S({"y/a.jpg": "M"}, burn=set(), opened={"y/a.jpg"}, closed={"x/a.jpg"}, reads=1),
    ],
    "copied file": [
        S({"a.jpg": "C"}),
        S({"a.jpg": "C", "b.jpg": "C"}, burn=set(), opened={"b.jpg"}, closed=set(), reads=1),
    ],
    "deleted file": [
        S({"a.jpg": "D", "b.jpg": "K"}),
        S({"b.jpg": "K"}, burn=set(), opened=set(), closed={"a.jpg"}, reads=0),
    ],
    "directory moved": [
        S({"d1/1.jpg": "A1", "d1/2.jpg": "A2", "d1/s/3.jpg": "A3"}),
        S({"d2/1.jpg": "A1", "d2/2.jpg": "A2", "d2/s/3.jpg": "A3"}, burn=set(),
          opened={"d2/1.jpg", "d2/2.jpg", "d2/s/3.jpg"},
          closed={"d1/1.jpg", "d1/2.jpg", "d1/s/3.jpg"}, reads=3),
    ],
    "always identical": [
        S({"a.jpg": "I", "b/a.jpg": "I"}, burn={"I"}),
        S({"a.jpg": "I", "b/a.jpg": "I"}, burn=set(), opened=set(), closed=set(), reads=0),
    ],
    "mtime changed, content didn't": [
        S({"a.jpg": F("T", mtime=1_600_000_000)}),
        S({"a.jpg": F("T", mtime=1_700_000_000)}, burn=set(),
          opened={"a.jpg"}, closed={"a.jpg"}, reads=1),
    ],
    "stealth edit, default (missed by design)": [
        S({"a.jpg": F("Q", size=4000, mtime=1_600_000_000)}),
        S({"a.jpg": F("Q2", size=4000, mtime=1_600_000_000)}, burn=set(),
          opened=set(), closed=set(), reads=0),
    ],
    "stealth edit, --rehash": [
        S({"a.jpg": F("Q", size=4000, mtime=1_600_000_000)}),
        S({"a.jpg": F("Q2", size=4000, mtime=1_600_000_000)}, rehash=True,
          burn={"Q2"}, opened={"a.jpg"}, closed={"a.jpg"}, reads=1),
    ],
    # beyond the ticket's list
    "unburned carryover": [
        S({"a.jpg": "P", "b.jpg": "R"}, burn={"P", "R"}, burned={"P"}),
        S({"a.jpg": "P", "b.jpg": "R"}, burn={"R"}, opened=set(), closed=set(), reads=0),
    ],
    "swap contents": [
        S({"a.jpg": "W1", "b.jpg": "W2"}),
        S({"a.jpg": "W2", "b.jpg": "W1"}, burn=set(),
          opened={"a.jpg", "b.jpg"}, closed={"a.jpg", "b.jpg"}, reads=2),
    ],
    "edit then revert": [
        S({"a.jpg": "V1"}),
        S({"a.jpg": "V2"}, burn={"V2"}),
        S({"a.jpg": "V1"}, burn=set(), opened={"a.jpg"}, closed={"a.jpg"}, reads=1),
    ],
}


def sym(hashes):
    return {TOKENS.get(h, h.hex()[:8]) for h in hashes}


def fmt(v):
    return "{" + ", ".join(sorted(v)) + "}" if isinstance(v, set) else str(v)


def run_case(name, steps, verbose):
    arc, ok = Archive(), True
    with tempfile.TemporaryDirectory(prefix="bdar-corpus-") as tmp:
        root = os.path.join(tmp, "lib")
        for i, step in enumerate(steps):
            write_tree(root, step.tree)
            got = arc.index(root, rehash=step.rehash)
            got["burn"] = sym(got["burn"])
            bad = [k for k, want in step.expect.items() if want is not None and got[k] != want]
            ok &= not bad
            if verbose or bad:
                tree = {p: as_file(v).token for p, v in step.tree.items()}
                print(f"    state {i}: {tree}{'  (rehash)' if step.rehash else ''}")
                for k, v in got.items():
                    want = step.expect[k]
                    mark = "" if want is None else ("ok" if v == want else f"WANT {fmt(want)}")
                    print(f"      {k:7} {fmt(v):40} {mark}")
            burned = got["burn"] if step.burned is None else step.burned
            arc.burn({h for h, t in TOKENS.items() if t in burned})
    print(f"{'PASS' if ok else 'FAIL'}  {name}")
    return ok


# ---- bulk: sparse tree for packer tests -------------------------------------

KINDS = [  # (ext, share, median bytes, sigma)
    ("jpg", 0.70, 4e6, 0.5),
    ("cr2", 0.20, 25e6, 0.2),
    ("mp4", 0.10, 150e6, 1.2),
]


def bulk(out, n, dup_rate, seed, oversize):
    rng = random.Random(seed)
    os.makedirs(out)
    total, written, dups = 0, [], 0
    for i in range(n):
        ext, med, sig = rng.choices([(k[0], k[2], k[3]) for k in KINDS],
                                    [k[1] for k in KINDS])[0]
        year = 2005 + i * 20 // n
        d = os.path.join(out, str(year), f"{year}-{1 + i % 12:02}-event{i // 40:03}")
        os.makedirs(d, exist_ok=True)
        path = os.path.join(d, f"IMG_{i:05}.{ext}")
        if written and rng.random() < dup_rate:
            src = rng.choice(written)
            size = os.path.getsize(src)
            with open(src, "rb") as fh, open(path, "wb") as out_fh:
                out_fh.write(fh.readline())  # same blob, new path: exercises dedup
                out_fh.truncate(size)  # copyfile would fill the hole
            dups += 1
        else:
            size = int(rng.lognormvariate(0, sig) * med)
            if i < oversize:
                size = 60 * 10**9  # bigger than any disc: exercises skip-and-report
            with open(path, "wb") as fh:
                fh.write(f"bdar-corpus {seed} {i}\n".encode())
                fh.truncate(size)  # the rest is a hole
            written.append(path)
        total += size
    du = sum(os.stat(os.path.join(d, f)).st_blocks * 512
             for d, _, fs in os.walk(out) for f in fs)
    print(f"{n} files ({dups} dup paths) in {out}")
    print(f"apparent {total / 2**30:.2f} GiB, on disk {du / 2**20:.1f} MiB")
    print(f"~{total / 41.5e9:.1f} BD-R DL discs, ~{total / 3.9e9:.1f} DVD discs (rough payloads)")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("cases")
    c.add_argument("-v", "--verbose", action="store_true")
    c.add_argument("names", nargs="*")
    b = sub.add_parser("bulk")
    b.add_argument("out")
    b.add_argument("--files", type=int, default=500)
    b.add_argument("--dup-rate", type=float, default=0.08)
    b.add_argument("--seed", type=int, default=1)
    b.add_argument("--oversize", type=int, default=0)
    a = ap.parse_args()
    if a.cmd == "bulk":
        return bulk(a.out, a.files, a.dup_rate, a.seed, a.oversize)
    picked = {k: v for k, v in CASES.items() if not a.names or k in a.names}
    results = [run_case(k, v, a.verbose) for k, v in picked.items()]
    sys.exit(0 if all(results) else 1)


if __name__ == "__main__":
    main()
