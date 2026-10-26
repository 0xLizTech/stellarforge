#!/usr/bin/env python3
"""Expand the compact issue sources in scripts/issues/src/*.txt into issues.md.

Source line format (one issue per line):
    title | areas | difficulty | summary | req; req; req | acceptance; acceptance

Directives at the top of each source file:
    #STAGE <name>     #PHASE <label>     #CONTEXT <text>     #OOS <text>

Usage:  python scripts/issues/build_issues.py
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SRC = HERE / "src"
OUT = HERE.parent.parent / "issues.md"

DIFFICULTY = {
    "gfi": ["good first issue"],
    "med": ["difficulty: medium"],
    "hard": ["difficulty: hard", "help wanted"],
}

AREA_LABEL = {
    "contracts": "contracts",
    "sdk": "sdk",
    "docs": "documentation",
    "ci": "ci",
    "security": "security",
    "testing": "testing",
    "tooling": "tooling",
    "frontend": "frontend",
    "research": "research",
}

DOD = {
    "contracts": [
        "`cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass",
        "New public functions have rustdoc, tests and an emitted event where state changes",
    ],
    "sdk": [
        "`npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`",
        "Public exports have TSDoc and the SDK README is updated if the API changed",
    ],
    "docs": ["Markdown renders correctly and all links resolve"],
    "ci": ["The workflow runs green on a pull request and uses pinned action versions"],
    "security": ["Findings, assumptions and residual risks are written down in `docs/`"],
    "testing": ["New tests fail when the behavior they guard is broken"],
    "tooling": ["The tool runs from a clean checkout and its usage is documented"],
    "frontend": ["The UI works on testnet, is keyboard accessible and has no console errors"],
    "research": ["Findings are written up in `docs/` with sources, tradeoffs and a recommendation"],
}

REFS = {
    "contracts": ["`contracts/`", "[ADR index](docs/architecture/)", "[Security notes](docs/SECURITY.md)"],
    "sdk": ["`sdk/src/`", "[SDK README](sdk/README.md)"],
    "docs": ["[docs/](docs/)", "[CONTRIBUTING.md](CONTRIBUTING.md)"],
    "ci": ["`.github/workflows/`"],
    "security": ["[Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)", "[Security notes](docs/SECURITY.md)"],
    "testing": ["`contracts/*/tests/`", "`sdk/tests/`"],
    "tooling": ["`scripts/`", "[Makefile](Makefile)"],
    "frontend": ["[ROADMAP.md](ROADMAP.md)"],
    "research": ["[ROADMAP.md](ROADMAP.md)", "[PRD](docs/prd/PRODUCT_REQUIREMENTS.md)"],
}


def split(s):
    return [x.strip() for x in s.split(";") if x.strip()]


def parse():
    stages = []
    for path in sorted(SRC.glob("*.txt")):
        st = {"issues": [], "file": path.name}
        for n, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            line = raw.strip()
            if not line:
                continue
            if line.startswith("#"):
                key, _, val = line[1:].partition(" ")
                st[key.lower()] = val.strip()
                continue
            parts = [p.strip() for p in line.split(" | ")]
            if len(parts) != 6:
                sys.exit(f"{path.name}:{n}: expected 6 fields, got {len(parts)}")
            title, areas, diff, summary, reqs, acc = parts
            if diff not in DIFFICULTY:
                sys.exit(f"{path.name}:{n}: bad difficulty {diff!r}")
            area_list = [a.strip() for a in areas.split(",")]
            for a in area_list:
                if a not in AREA_LABEL:
                    sys.exit(f"{path.name}:{n}: unknown area {a!r}")
            st["issues"].append(
                dict(title=title, areas=area_list, diff=diff, summary=summary,
                     reqs=split(reqs), acc=split(acc))
            )
        stages.append(st)
    return stages


def render(st, it):
    labels = [AREA_LABEL[a] for a in it["areas"]] + [st["phase"]] + DIFFICULTY[it["diff"]]
    dod, refs = [], []
    for a in it["areas"]:
        dod += [d for d in DOD[a] if d not in dod]
        refs += [r for r in REFS[a] if r not in refs]
    out = [f"## {it['title']}", "", f"**Labels:** {', '.join(labels)}", "", "### Summary", "", it["summary"], "",
           "### Context", "", st["context"], "", f"This issue belongs to **{st['stage']}**.", "",
           "### Objective", "", f"Deliver the change described above so that: {it['acc'][0][0].lower() + it['acc'][0][1:]}.", "",
           "### Requirements", ""]
    out += [f"- [ ] {r}" for r in it["reqs"]]
    out += ["", "### Out of Scope", "", st["oos"], "", "### Acceptance Criteria", ""]
    out += [f"- [ ] {a}" for a in it["acc"]]
    out += ["- [ ] CI is green and a maintainer has reviewed the change", "",
            "### Definition of Done", ""]
    out += [f"- [ ] {d}" for d in dod]
    out += ["- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes", "",
            "### References", ""]
    out += [f"- {r}" for r in refs]
    out += ["", "---", ""]
    return "\n".join(out)


def main():
    stages = parse()
    total = sum(len(s["issues"]) for s in stages)
    titles = [i["title"] for s in stages for i in s["issues"]]
    dupes = {t for t in titles if titles.count(t) > 1}
    if dupes:
        sys.exit(f"duplicate titles: {sorted(dupes)}")
    parts = ["# StellarForge Open Source Issues", "",
             f"{total} issues covering the development stages of the project, derived from "
             "[ROADMAP.md](ROADMAP.md), the [PRD](docs/prd/PRODUCT_REQUIREMENTS.md) and the "
             "[Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md).", "",
             "Publish them with `python scripts/issues/push_issues.py` (see `scripts/issues/README.md`).",
             "Do not edit titles after publishing: the script uses them to skip issues that already exist.", "",
             "## Stages", ""]
    for s in stages:
        parts.append(f"- {s['stage']} ({len(s['issues'])} issues)")
    parts += ["", "---", ""]
    for s in stages:
        parts.append(f"<!-- {s['stage']} -->\n")
        for it in s["issues"]:
            parts.append(render(s, it))
    OUT.write_text("\n".join(parts), encoding="utf-8", newline="\n")
    print(f"wrote {OUT} with {total} issues")


if __name__ == "__main__":
    main()
