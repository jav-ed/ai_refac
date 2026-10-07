#!/usr/bin/env python3
"""Count visible filesystem entries and direct children for a structural review."""

import argparse
import json
import os
from pathlib import Path
import stat
import subprocess
import sys


def git_command(directory, arguments, input_bytes=None):
    result = subprocess.run(
        ["git", "-C", str(directory), *arguments],
        input=input_bytes,
        capture_output=True,
    )
    return result


def directory_entries(directory, include_hidden, include_ignored):
    with os.scandir(directory) as stream:
        entries = sorted(
            (
                entry
                for entry in stream
                if entry.name != ".git"
                and (include_hidden or not entry.name.startswith("."))
            ),
            key=lambda entry: entry.name,
        )

    if include_ignored or not entries:
        return entries

    # NUL-delimited paths preserve filenames containing whitespace and newlines.
    names = [
        os.fsencode(entry.name)
        + (b"/" if entry.is_dir(follow_symlinks=False) else b"")
        for entry in entries
    ]
    result = git_command(
        directory,
        ["check-ignore", "--no-index", "--stdin", "-z"],
        b"\0".join(names) + b"\0",
    )
    if result.returncode not in (0, 1):
        detail = os.fsdecode(result.stderr).strip()
        raise RuntimeError(f"Cannot evaluate Git ignore rules in {directory}: {detail}")
    ignored = set(result.stdout.split(b"\0"))
    return [entry for entry, name in zip(entries, names) if name not in ignored]


def count_tree(root, recursive, include_hidden, include_ignored):
    if not include_ignored:
        result = git_command(root, ["rev-parse", "--is-inside-work-tree"])
        if result.returncode or result.stdout.strip() != b"true":
            detail = os.fsdecode(result.stderr).strip() or "Git did not report a working tree."
            raise RuntimeError(
                f"Cannot establish a Git working tree for {root}: {detail} "
                "Use --include-ignored for explicit unfiltered filesystem counts."
            )

    rows = []
    pending = [root]
    while pending:
        directory = pending.pop()
        counts = {"files": 0, "folders": 0, "symlinks": 0, "other": 0}
        children = []
        for entry in directory_entries(directory, include_hidden, include_ignored):
            # Links are counted separately; never traverse into their targets.
            mode = entry.stat(follow_symlinks=False).st_mode
            if stat.S_ISLNK(mode):
                counts["symlinks"] += 1
            elif stat.S_ISDIR(mode):
                counts["folders"] += 1
                children.append(Path(entry.path))
            elif stat.S_ISREG(mode):
                counts["files"] += 1
            else:
                counts["other"] += 1
        rows.append({"path": directory.relative_to(root).as_posix(), **counts})
        if recursive:
            pending.extend(reversed(children))

    return {
        "root": str(root),
        "recursive": recursive,
        "include_hidden": include_hidden,
        "git_ignore_patterns": not include_ignored,
        "git_metadata_excluded": True,
        "follow_symlinks": False,
        "totals": {
            kind: sum(row[kind] for row in rows)
            for kind in ("files", "folders", "symlinks", "other")
        },
        "directories": rows,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", help="Directory whose contents should be counted.")
    parser.add_argument("--recursive", action="store_true", help="Count the complete subtree.")
    parser.add_argument("--all", action="store_true", help="Include hidden files and folders.")
    parser.add_argument(
        "--include-ignored", action="store_true", help="Count without Git ignore filtering."
    )
    arguments = parser.parse_args()
    try:
        root = Path(arguments.path).resolve(strict=True)
        if not root.is_dir():
            raise RuntimeError(f"Expected a directory to count, received: {root}")
        report = count_tree(root, arguments.recursive, arguments.all, arguments.include_ignored)
        # Publish only a complete result; filesystem or Git failures leave stdout empty.
        print(json.dumps(report, indent=2))
    except (OSError, RuntimeError) as error:
        print(f"File-tree count failed for {arguments.path!r}: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
