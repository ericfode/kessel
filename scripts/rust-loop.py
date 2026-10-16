#!/usr/bin/env python3
"""Run Cargo once; preserve full output locally and print a bounded receipt."""
import collections
import datetime
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import time


def totals(paths):
    rows = []
    pattern = re.compile(
        r"^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; "
        r"(\d+) ignored; (\d+) measured; (\d+) filtered out"
    )
    for path in paths:
        with path.open(errors="replace") as stream:
            for line in stream:
                match = pattern.match(line)
                if match:
                    rows.append(tuple(map(int, match.groups())))
    if not rows:
        return "tests: unavailable (no completed libtest summaries)"
    passed, failed, ignored, measured, filtered = map(sum, zip(*rows))
    return (f"tests: {passed + failed + ignored} total; {passed} passed; "
            f"{failed} failed; {ignored} ignored; {filtered} filtered; {measured} measured")


def failure_excerpt(paths):
    # Bound both bytes read and output; never reread cumulative logs while running.
    parts = []
    truncated = False
    for path in paths:
        with path.open("rb") as stream:
            size = stream.seek(0, os.SEEK_END)
            truncated |= size > 6000
            stream.seek(max(0, size - 6000))
            tail = stream.read(6000).decode("utf-8", errors="replace")
        parts.append(f"--- {path.name} (tail) ---\n{tail}")
    excerpt = "\n".join(parts)
    truncated |= len(excerpt.splitlines()) > 60
    excerpt = "\n".join(collections.deque(excerpt.splitlines(), maxlen=60))
    truncated |= len(excerpt.encode("utf-8")) > 6000
    return excerpt.encode("utf-8")[-6000:].decode("utf-8", errors="ignore"), truncated


def main():
    root = Path(__file__).resolve().parent.parent
    os.chdir(root)
    logroot = root / "target" / "rust-loop-logs"
    logroot.mkdir(parents=True, exist_ok=True)
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ-")
    directory = Path(tempfile.mkdtemp(prefix=stamp, dir=logroot))
    paths = [directory / "stdout.log", directory / "stderr.log"]
    jobs = os.environ.get("KESSEL_BUILD_JOBS", "2")
    threads = os.environ.get("KESSEL_TEST_THREADS", "4")
    cmd = ["cargo", "test", "--offline", "--locked", "--profile", "rust-loop",
           "--workspace", "--lib", "--bins", "--test", "token_attack_admission",
           "--jobs", jobs, "--", "--test-threads=" + threads, *sys.argv[1:]]
    revision = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True,
                              text=True, check=False).stdout.strip() or "unavailable"
    started = time.monotonic()
    with paths[0].open("wb") as stdout, paths[1].open("wb") as stderr:
        if jobs not in tuple(map(str, range(1, 9))) or threads not in tuple(map(str, range(1, 9))):
            stderr.write(b"Rust loop concurrency must be an integer from 1 through 8\n")
            code = 2
        else:
            try:
                code = subprocess.run(cmd, stdout=stdout, stderr=stderr, check=False).returncode
                if code < 0:
                    code = 128 - code  # Preserve normal shell signal-exit semantics.
            except OSError as error:
                stderr.write((str(error) + "\n").encode())
                code = 127
    try:
        receipt = (f"command: {shlex.join(cmd)}\nrevision: {revision}\n"
                   f"elapsed: {time.monotonic() - started:.2f}s\n{totals(paths)}\n"
                   f"exit: {code}\nlogs: {directory}\n")
        if code:
            excerpt, truncated = failure_excerpt(paths)
            receipt += (f"excerpt truncated: {'yes' if truncated else 'no'}\n"
                        "failure: Cargo or runner failed; excerpt below is bounded to "
                        "60 lines / 6000 bytes. "
                        "Complete stdout/stderr are in the logs.\n" + excerpt + "\n")
        (directory / "summary.txt").write_text(receipt)
        print(receipt, end="", file=sys.stderr if code else sys.stdout)
    except Exception as error:
        # A reporting failure must not replace an existing Cargo failure status.
        print(f"command: {shlex.join(cmd)}\nrevision: {revision}\n"
              f"elapsed: {time.monotonic() - started:.2f}s\n"
              f"tests: unavailable (reporting error: {error})\n"
              f"exit: {code or 1}\nlogs: {directory}", file=sys.stderr)
        return code or 1
    return code


if __name__ == "__main__":
    sys.exit(main())
