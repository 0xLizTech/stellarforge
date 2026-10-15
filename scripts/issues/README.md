# Issue tooling

`issues.md` (repo root) holds 500 open source issues covering the project's development stages.

| File | Purpose |
|---|---|
| `src/*.txt` | Compact source, one issue per line, grouped by stage |
| `build_issues.py` | Expands `src/` into `issues.md` |
| `push_issues.py` | Creates the issues on GitHub from `issues.md` |

## Publish

1. Create a token at https://github.com/settings/tokens with the `repo` scope (fine-grained: Issues read and write).
2. Export it and do a dry run:

```bash
export GITHUB_TOKEN=ghp_xxx          # PowerShell: $env:GITHUB_TOKEN = "ghp_xxx"
python scripts/issues/push_issues.py
```

3. Create a few first, check them on GitHub, then the rest:

```bash
python scripts/issues/push_issues.py --apply --limit 5
python scripts/issues/push_issues.py --apply
```

The script creates missing labels, skips titles that already exist, sleeps between requests and
stops on the first error. Re-running continues where it stopped. It needs only Python 3.8+.

## Edit

Change `src/*.txt`, then run `python scripts/issues/build_issues.py`. Do not change titles after publishing.
