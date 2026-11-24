#!/usr/bin/env python3
"""Create GitHub issues from issues.md.

Standard library only. Dry run by default.

    export GITHUB_TOKEN=ghp_...           # needs `repo` (or Issues: write) scope
    python scripts/issues/push_issues.py                    # dry run, prints what would happen
    python scripts/issues/push_issues.py --apply            # create issues
    python scripts/issues/push_issues.py --apply --limit 10 # create the first 10 only

The script is safe to re-run: it skips titles that already exist in the repo.
"""
import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
API = "https://api.github.com"

LABEL_COLORS = {
    "contracts": "5319e7", "sdk": "0e8a16", "documentation": "0075ca", "ci": "fbca04",
    "security": "b60205", "testing": "c2e0c6", "tooling": "bfd4f2", "frontend": "d4c5f9",
    "research": "ededed", "phase-1": "1d76db", "phase-2": "1d76db", "phase-3": "1d76db",
    "phase-4": "1d76db", "phase-5": "1d76db", "good first issue": "7057ff",
    "help wanted": "008672", "difficulty: medium": "fef2c0", "difficulty: hard": "d93f0b",
}


def detect_repo():
    import subprocess
    try:
        url = subprocess.check_output(["git", "remote", "get-url", "origin"], cwd=ROOT, text=True).strip()
    except Exception:
        return None
    m = re.search(r"github\.com[:/]([^/]+/[^/.]+)", url)
    return m.group(1) if m else None


def parse(path):
    issues, cur = [], None
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("## ") and not line.startswith("## Stages"):
            cur = {"title": line[3:].strip(), "lines": []}
            issues.append(cur)
        elif cur is not None:
            cur["lines"].append(line)
    out = []
    for it in issues:
        body = "\n".join(it["lines"]).strip()
        body = re.sub(r"\n---\s*$", "", body).strip()
        m = re.match(r"\*\*Labels:\*\*\s*(.+)", body)
        labels = [x.strip() for x in m.group(1).split(",")] if m else []
        if m:
            body = body[m.end():].strip()
        body = re.sub(r"<!--.*?-->", "", body, flags=re.S).strip()
        out.append({"title": it["title"], "labels": labels, "body": body})
    return out


class Gh:
    def __init__(self, repo, token, use_cli=False):
        self.repo, self.token, self.use_cli = repo, token, use_cli

    def call(self, method, path, data=None):
        if self.use_cli:
            return self.call_cli(method, path, data)
        try:
            return self.call_http(method, path, data)
        except urllib.error.URLError as e:
            if "CERTIFICATE_VERIFY_FAILED" not in str(e):
                raise
            print("  Python SSL verification failed; switching to the gh CLI")
            self.use_cli = True
            return self.call_cli(method, path, data)

    def call_cli(self, method, path, data=None):
        """Use the gh CLI, which trusts the OS certificate store."""
        import subprocess
        for attempt in range(5):
            p = subprocess.run(["gh", "api", "-X", method, "-i", path.lstrip("/"), "--input", "-"],
                               input=json.dumps(data) if data is not None else "",
                               capture_output=True, text=True, encoding="utf-8")
            raw = p.stdout.replace("\r\n", "\n")
            head, _, body = raw.partition("\n\n")
            m = re.match(r"HTTP/\S+ (\d+)", head)
            if not m:
                sys.exit(f"gh api failed: {p.stderr.strip() or raw[:300]}")
            status = int(m.group(1))
            if status in (403, 429) and ("rate limit" in body.lower() or "secondary" in body.lower()):
                wait = 60 * (attempt + 1)
                print(f"  rate limited, sleeping {wait}s")
                time.sleep(wait)
                continue
            if status >= 500:
                time.sleep(5 * (attempt + 1))
                continue
            try:
                parsed = json.loads(body) if body.strip() else None
            except json.JSONDecodeError:
                parsed = body
            return status, parsed, {}
        sys.exit("giving up after repeated failures")

    def call_http(self, method, path, data=None):
        req = urllib.request.Request(
            API + path, method=method,
            data=json.dumps(data).encode() if data is not None else None,
            headers={"Authorization": f"Bearer {self.token}", "Accept": "application/vnd.github+json",
                     "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "stellarforge-issue-pusher",
                     "Content-Type": "application/json"})
        for attempt in range(5):
            try:
                with urllib.request.urlopen(req) as r:
                    return r.status, json.loads(r.read() or b"null"), r.headers
            except urllib.error.HTTPError as e:
                body = e.read().decode()
                if e.code in (403, 429) and ("rate limit" in body.lower() or "secondary" in body.lower()
                                             or e.headers.get("Retry-After")):
                    wait = int(e.headers.get("Retry-After") or 60 * (attempt + 1))
                    print(f"  rate limited, sleeping {wait}s")
                    time.sleep(wait)
                    continue
                if e.code >= 500:
                    time.sleep(5 * (attempt + 1))
                    continue
                return e.code, body, e.headers
        sys.exit("giving up after repeated failures")

    def paged(self, path):
        page = 1
        while True:
            sep = "&" if "?" in path else "?"
            status, data, _ = self.call("GET", f"{path}{sep}per_page=100&page={page}")
            if status != 200:
                sys.exit(f"GET {path} failed: {status} {data}")
            if not data:
                return
            yield from data
            page += 1


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--file", default=str(ROOT / "issues.md"))
    ap.add_argument("--repo", default=os.environ.get("GITHUB_REPOSITORY") or detect_repo(),
                    help="owner/name (default: from git remote origin)")
    ap.add_argument("--apply", action="store_true", help="actually create issues (default is dry run)")
    ap.add_argument("--limit", type=int, default=0, help="stop after creating N issues")
    ap.add_argument("--skip", type=int, default=0, help="skip the first N issues in the file")
    ap.add_argument("--delay", type=float, default=2.0, help="seconds between creations (avoid abuse limits)")
    ap.add_argument("--gh", action="store_true",
                    help="send requests through the gh CLI (use when Python SSL verification fails)")
    args = ap.parse_args()

    issues = parse(Path(args.file))
    print(f"parsed {len(issues)} issues from {args.file}")
    if not args.repo:
        sys.exit("cannot detect repo; pass --repo owner/name")
    print(f"target repo: {args.repo}")

    if not args.apply:
        for it in issues[args.skip:args.skip + (args.limit or len(issues))]:
            print(f"[dry-run] {it['title']}  [{', '.join(it['labels'])}]")
        print("\nDry run only. Re-run with --apply to create issues.")
        return

    token = os.environ.get("GITHUB_TOKEN")
    if not token and not args.gh:
        sys.exit("set GITHUB_TOKEN first, or pass --gh to use a logged-in gh CLI")
    gh = Gh(args.repo, token, use_cli=args.gh)

    existing_labels = {l["name"] for l in gh.paged(f"/repos/{args.repo}/labels")}
    needed = {l for it in issues for l in it["labels"]}
    for name in sorted(needed - existing_labels):
        status, data, _ = gh.call("POST", f"/repos/{args.repo}/labels",
                                  {"name": name, "color": LABEL_COLORS.get(name, "ededed")})
        print(f"label {name}: {'created' if status == 201 else f'failed {status} {data}'}")

    existing = {i["title"].strip().lower() for i in gh.paged(f"/repos/{args.repo}/issues?state=all")
                if "pull_request" not in i}
    print(f"{len(existing)} issues already exist in the repo")

    created = skipped = 0
    for it in issues[args.skip:]:
        if it["title"].strip().lower() in existing:
            skipped += 1
            continue
        status, data, _ = gh.call("POST", f"/repos/{args.repo}/issues",
                                  {"title": it["title"], "body": it["body"], "labels": it["labels"]})
        if status != 201:
            print(f"FAILED {it['title']}: {status} {data}")
            print(f"stopping. created={created} skipped={skipped}. Re-run to continue.")
            sys.exit(1)
        created += 1
        print(f"created #{data['number']}: {it['title']}")
        if args.limit and created >= args.limit:
            break
        time.sleep(args.delay)
    print(f"done. created={created} skipped(existing)={skipped}")


if __name__ == "__main__":
    main()
