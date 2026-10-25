#!/usr/bin/env python3
"""
PROTOTYPE -- throwaway. Answers "Disc packing: hitting the ECC input ceiling".

Question: packing lazily, one disc at a time, in path order, how close to
max_payload_bytes can we land, how many exact `xorriso -print-size` calls
does it take, and how wrong is a cheap size estimate?

The exact size comes from real xorriso, run against a sparse mirror of each
disc's files (same names, same sizes, no data), so it needs no access to the
library itself -- only a listing.

usage: pack.py LISTING.sqlite --root R [--root R ...] [--discs N] [--gapfill WINDOW]

LISTING is a sqlite db with table files(name, type, size): archive-relative
path, 'file' or 'dir', bytes.
"""
import argparse, os, shutil, sqlite3, subprocess, tempfile

SECTOR = 2048
# bdr-dl: capacity / 1.2052 (20% RS02), floored to a sector
MAX_PAYLOAD = (50_050_629_632 * 10_000 // 12_052) // SECTOR * SECTOR
EXCLUDE_EXT = (".thm", ".tmp")

# estimate model, fitted from the calibration run (see --calibrate)
PER_FILE = 202
PER_DIR = 3_427
FIXED = 343_412
MARGIN = 1_000_000  # > worst residual seen (185 KB)


def load(db, roots):
    rows = sqlite3.connect(db).execute(
        "select name, size from files where type = 'file' order by name")
    return [(n, s) for n, s in rows
            if n.startswith(roots) and not n.lower().endswith(EXCLUDE_EXT)]


def dirs_of(files):
    out = set()
    for name, _ in files:
        parts = name.split("/")[:-1]
        for i in range(1, len(parts) + 1):
            out.add("/".join(parts[:i]))
    return out


def roundup(n):
    return -(-n // SECTOR) * SECTOR


def estimate(files):
    data = sum(roundup(s) for _, s in files)
    return FIXED + data + PER_FILE * len(files) + PER_DIR * len(dirs_of(files))


def exact(files):
    """Real `xorriso -print-size` over a sparse mirror of `files`."""
    root = tempfile.mkdtemp(prefix="bdar-proto-")
    try:
        for name, size in files:
            p = os.path.join(root, name)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, "wb") as f:
                f.truncate(size)
        out = subprocess.run(
            ["xorrisofs", "-print-size", "-quiet", "-iso-level", "3",
             "-r", "-J", "-joliet-long", "-V", "BDAR_0001", root],
            capture_output=True, text=True, check=True).stdout
        return int(out.split()[-1]) * SECTOR
    finally:
        shutil.rmtree(root)


def plan_disc(remaining, gapfill):
    """Next disc: longest path-order prefix whose estimate fits under
    MAX_PAYLOAD - MARGIN, then optionally fill the leftover from the next
    `gapfill` files. One exact check; trim and recheck only on overshoot."""
    budget = MAX_PAYLOAD - MARGIN - FIXED
    used, dirs, chosen, i = 0, set(), [], 0
    def cost(f):
        new_dirs = [d for d in parents(f[0]) if d not in dirs]
        return roundup(f[1]) + PER_FILE + PER_DIR * len(new_dirs), new_dirs
    while i < len(remaining):
        c, nd = cost(remaining[i])
        if used + c > budget:
            break
        used += c; dirs.update(nd); chosen.append(remaining[i]); i += 1
    rest = remaining[i:]
    filled = 0
    if gapfill and rest:
        keep = []
        for f in rest[:gapfill]:
            c, nd = cost(f)
            if used + c <= budget:
                used += c; dirs.update(nd); chosen.append(f); filled += 1
            else:
                keep.append(f)
        rest = keep + rest[gapfill:]
    size = exact(chosen); calls = 1
    while size > MAX_PAYLOAD:
        f = chosen.pop(); rest.insert(0, f)
        size = exact(chosen); calls += 1
    return chosen, rest, size, calls, filled


def parents(name):
    parts = name.split("/")[:-1]
    return ["/".join(parts[:i]) for i in range(1, len(parts) + 1)]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("listing")
    ap.add_argument("--discs", type=int, default=999)
    ap.add_argument("--gapfill", type=int, default=0)
    ap.add_argument("--calibrate", action="store_true")
    ap.add_argument("--root", action="append", required=True,
                    help="archive-relative root prefix, repeatable")
    a = ap.parse_args()
    files = load(a.listing, tuple(r.rstrip('/') + '/' for r in a.root))
    total = sum(s for _, s in files)
    print(f"{len(files)} files, {total/1e12:.3f} TB, "
          f"max_payload {MAX_PAYLOAD:,} -> lower bound "
          f"{-(-total // MAX_PAYLOAD)} discs")

    if a.calibrate:
        calibrate(files)
        return

    rest, n, waste = files, 0, 0
    print(f"{'disc':>4} {'files':>6} {'dirs':>5} {'estimate':>15} {'exact':>15} "
          f"{'est-exact':>10} {'unused':>13} calls filled")
    while rest and n < a.discs:
        n += 1
        chosen, rest, size, calls, filled = plan_disc(rest, a.gapfill)
        est = estimate(chosen)
        unused = MAX_PAYLOAD - size
        if rest:
            waste += unused
        print(f"{n:>4} {len(chosen):>6} {len(dirs_of(chosen)):>5} {est:>15,} "
              f"{size:>15,} {est - size:>10,} {unused:>13,} {calls:>5} {filled}")
    print(f"discs {n}, unused on full discs {waste/1e9:.2f} GB "
          f"({waste / (MAX_PAYLOAD * max(n - 1, 1)):.3%})")


def calibrate(files):
    """Fit FIXED/PER_FILE/PER_DIR from exact sizes of path-order slices."""
    import random
    random.seed(1)
    pts = []
    for _ in range(12):
        start = random.randrange(len(files) - 20000)
        k = random.choice([50, 500, 2000, 8000, 20000])
        sl = files[start:start + k]
        data = sum(roundup(s) for _, s in sl)
        ov = exact(sl) - data
        pts.append((len(sl), len(dirs_of(sl)), ov))
        print(f"files {len(sl):>6} dirs {len(dirs_of(sl)):>5} overhead {ov:>12,} "
              f"= {ov/len(sl):.0f} B/file")
    # least squares ov = F + a*files + b*dirs
    import numpy as np
    A = np.array([[1, f, d] for f, d, _ in pts], float)
    y = np.array([o for *_, o in pts], float)
    (F, pf, pd), *_ = np.linalg.lstsq(A, y, rcond=None)
    print(f"FIXED={F:.0f} PER_FILE={pf:.1f} PER_DIR={pd:.1f}")
    for f, d, o in pts:
        print(f"  resid {o - (F + pf*f + pd*d):>10,.0f}")


if __name__ == "__main__":
    main()
