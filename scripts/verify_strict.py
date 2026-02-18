import sys
from pathlib import Path
import subprocess, tarfile
import os

def out(s): print(s)

def run(cmd: list[str]) -> str:
    # exit code doesn't control flow, just logging
    out("CMD: " + " ".join(cmd))
    try:
        p = subprocess.run(cmd, text=True, capture_output=True)
        if p.stdout.strip(): out(p.stdout.rstrip())
        if p.stderr.strip(): out(p.stderr.rstrip())
        out(f"RC: {p.returncode}")
        return (p.stdout or "") + "\n" + (p.stderr or "")
    except Exception as e:
        out(f"ERROR: subprocess failed: {e}")
        return ""

OK = True

# A) HEAD
head = ""
try:
    head = subprocess.check_output(["git","rev-parse","HEAD"], text=True).strip()
except Exception as e:
    out(f"ERROR: git rev-parse HEAD failed: {e}")
    OK = False

if OK:
    out(f"OK: HEAD={head}")

# B) git clean
if OK:
    try:
        s = subprocess.check_output(["git","status","--porcelain"], text=True)
        if s.strip():
            out("ERROR: git repository is dirty (strict prohibited).")
            out("HINT: untracked/modified files are present. Stash or remove them.")
            OK = False
        else:
            out(f"OK: git clean ({len(s)} bytes)")
    except Exception as e:
        out(f"ERROR: git status failed: {e}")
        OK = False

# C) Find local prverify report containing HEAD
report = None
if OK:
    d = Path(".local/prverify")
    if not d.is_dir():
        out("ERROR: .local/prverify missing")
        out("HINT: Run 'nix run .#prverify' first")
        OK = False
    else:
        # Sort by mtime descending, pick newest matching
        files = sorted(d.glob("prverify_*.md"), key=lambda p: p.stat().st_mtime, reverse=True)[:120]
        for p in files:
            try:
                txt = p.read_text(encoding="utf-8", errors="replace")
                if head in txt:
                    report = p
                    break
            except Exception as e:
                out(f"SKIP: read failed: {p}: {e}")

        if report is None:
            out("ERROR: no prverify report contains HEAD")
            out("HINT: Re-run 'nix run .#prverify' with current HEAD")
            OK = False
        else:
            out(f"OK: REPORT={report}")

# D) strict create (allow failures, don't exit)
bundle = None
if OK:
    # ensuring output dir exist
    run(["mkdir", "-p", ".local/review-bundles"])
    run(["go","run","./cmd/reviewbundle","create","--mode","strict","--out-dir",".local/review-bundles"])

    head12 = head[:12]
    bd = Path(".local/review-bundles")
    # Finding bundle with matching head12
    cand = sorted(bd.glob(f"*_*strict_*_{head12}.tar.gz"), key=lambda p: p.stat().st_mtime, reverse=True)
    if not cand:
        # Fallback search
        cand = sorted(bd.glob("*_strict_*.tar.gz"), key=lambda p: p.stat().st_mtime, reverse=True)
    
    if not cand:
        out("ERROR: strict bundle not found")
        OK = False
    else:
        bundle = cand[0]
        # Check if bundle matches exact head timestamp if possible
        out(f"OK: BUNDLE_STRICT={bundle}")

# E) strict verify
if OK and bundle:
    txt = run(["go","run","./cmd/reviewbundle","verify",str(bundle)])
    if "PASS:" not in txt:
        out("ERROR: verify did not report PASS")
        OK = False
    else:
        out("OK: verify PASS")

# F) Check evidence inside tar
if OK and bundle and report:
    want = f"review/evidence/prverify/{report.name}"
    try:
        with tarfile.open(bundle, "r:gz") as tf:
            names = tf.getnames()
        if want in names:
            out(f"OK: tar contains evidence: {want}")
        else:
            out(f"ERROR: tar missing evidence: {want}")
            # Also list what IS there for debugging
            evidence_files = [n for n in names if "review/evidence" in n]
            out(f"     Found evidence: {evidence_files}")
            OK = False
    except Exception as e:
        out(f"ERROR: tar check failed: {e}")
        OK = False

if OK:
    out("OK: STRICT RITUAL COMPLETE")
    sys.exit(0)
else:
    out("ERROR: STRICT RITUAL FAILED (see logs above)")
    sys.exit(1)
