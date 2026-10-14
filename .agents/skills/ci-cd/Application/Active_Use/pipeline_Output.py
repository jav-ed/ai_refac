"""Read-only Woodpecker run viewer: recent pipelines, one pipeline's steps, or one step's log. GET requests only."""

import argparse
import base64
import json
import os
import urllib.request

BASE = "https://ci.zetunweb.com/api"


def get(path):
    request = urllib.request.Request(
        BASE + path,
        headers={
            "Authorization": "Bearer " + os.environ["WOODPECKER_API_TOKEN"],
            "Accept": "application/json",
        },
        method="GET",
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read())


def steps_of(detail):
    return [step for workflow in detail.get("workflows", []) for step in workflow.get("children", [])]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repo", help="OWNER/REPO as Forgejo names it")
    parser.add_argument("number", nargs="?", type=int, help="pipeline number; omit to list recent pipelines")
    parser.add_argument("step", nargs="?", help="step name such as 05_Deploy; omit to list the steps")
    parser.add_argument("--tail", type=int, default=40, help="log lines to print from the end (default 40)")
    args = parser.parse_args()

    repo_id = get("/repos/lookup/" + args.repo)["id"]
    if args.number is None:
        for pipeline in get(f"/repos/{repo_id}/pipelines?perPage=10"):
            subject = (pipeline.get("message") or "").splitlines()[:1]
            print(
                f"p{pipeline['number']:<5} {pipeline['status']:<9} {pipeline['event']:<5} "
                f"{pipeline['branch']:<6} {pipeline['commit'][:8]} {subject[0][:60] if subject else ''}"
            )
        return

    detail = get(f"/repos/{repo_id}/pipelines/{args.number}")
    steps = steps_of(detail)
    if args.step is None:
        print(f"p{args.number} {detail['status']} commit {detail['commit'][:8]}")
        for step in steps:
            print(f"  {step['name']:<16} {step['state']:<8} exit={step.get('exit_code')}")
        return

    step = next((item for item in steps if item["name"] == args.step), None)
    if step is None:
        raise SystemExit(f"No step named {args.step}; available: {', '.join(item['name'] for item in steps)}")
    entries = get(f"/repos/{repo_id}/logs/{args.number}/{step['id']}")
    # One API entry is one log line without its newline, and some entries carry no data.
    lines = [base64.b64decode(entry["data"]).decode(errors="replace") for entry in entries if entry.get("data")]
    text = "\n".join(lines).replace("\r", "\n").splitlines()
    print(f"p{args.number} {args.step} ({step['state']}, exit {step.get('exit_code')}), last {min(args.tail, len(text))} of {len(text)} lines")
    print("\n".join(text[-args.tail:]))


if __name__ == "__main__":
    main()
