# Branch Protection and Repository Settings

This document defines the branch protection rules and repository settings for
`0xLizTech/stellarforge`, and provides a checklist an admin applies when setting
up or auditing the repository. The workflow job names referenced here come from
[`.github/workflows/`](../.github/workflows/); if a workflow's job name changes,
update this document in the same pull request.

## Protected branches

| Branch | Role | Protection |
| --- | --- | --- |
| `main` | Release branch; always deployable to testnet | Strict — see below |
| `develop` | Integration branch, where used | Same required checks, 1 approval |

Direct pushes to protected branches are disabled. Every change lands via pull
request.

## Required status checks

All of these must pass before a pull request can merge to `main`. Checks come
from the pull-request triggers in the workflows listed.

| Check (job name) | Workflow | Runs on |
| --- | --- | --- |
| `Check & Lint` | Contracts CI | PR touching `contracts/**`, `Cargo.lock`, parity fixture |
| `Unit Tests` | Contracts CI | same paths as above |
| `Build WASM Artifacts` | Contracts CI | same paths as above |
| `Oracle parity fixture (Rust → JSON)` | SDK CI | PR touching `sdk/**`, oracle-adapter |
| `TypeScript Tests` | SDK CI | same paths as above |
| `Build Package` | SDK CI | same paths as above |
| `Cargo Audit (Rust advisory database)` | Security Audit | PR touching dependencies |
| `npm Audit (SDK runtime dependencies)` | Security Audit | same paths as above |
| `Dependency Review` | Security Audit | every PR (dependency changes) |

Notes:

- **Path-filter caveat.** Contracts CI, SDK CI and Security Audit only run for
  pull requests that touch their paths. GitHub leaves a required check pending
  when its workflow never ran, which would block unrelated PRs from merging.
  Recommended layout: add one always-running `PR Gate` workflow (a single
  echo-only job) and make it the only *ruleset-required* check; the checks in
  the table remain required **by policy** — a maintainer rejects a merge whose
  applicable checks did not pass. When a failed/missing check is the reason for
  a hold, say so in the review thread so the author can react prior to the
  quarterly settings audit.
- `testnet-smoke.yml` runs on a schedule, not on pull requests, so it is not a
  required check.

## Review requirements

- **1 approving review** minimum; `main` requires a review from a
  [CODEOWNER](../.github/CODEOWNERS).
- **Dismiss stale approvals** when new commits are pushed.
- **Require approval of the most recent push** (the pusher cannot self-approve).
- **Require conversation resolution** before merging.
- **Require branches to be up to date** before merging, so the merge is tested
  against the actual `main` target.

## History and safety rules

- **Linear history** required; squash or rebase only — no merge commits.
- **Allow force pushes: off. Allow deletions: off.**
- **Include administrators**: off by default, with a documented break-glass
  exception (see below). With it off, an admin can push a hotfix under incident
  if GitHub is degraded; the change still must be followed by a retroactive PR.
- **Automatically delete head branches** after merge (repository setting).

## Settings checklist

An admin verifies each item once per quarter, or whenever a workflow changes:

- [ ] `main` protection rule exists; "Require a pull request before merging" is on.
- [ ] "Require approvals" = 1; "Require review from Code Owners" is on.
- [ ] "Dismiss stale pull request approvals when new commits are pushed" is on.
- [ ] "Require approval of the most recent reviewable push" is on.
- [ ] "Require conversation resolution before merging" is on.
- [ ] "Require status checks to pass" is on; required checks follow the path-filter caveat (`PR Gate` as the ruleset-required check; the table enforced by policy).
- [ ] "Require branches to be up to date before merging" is on.
- [ ] "Require linear history" is on.
- [ ] Force pushes and branch deletion are disabled for `main`.
- [ ] "Automatically delete head branches" is on (repository → General).
- [ ] Dependabot alerts and security updates are enabled.
- [ ] Actions are restricted to the workflows in `.github/workflows/` (or the
      org policy equivalent).

Verify the live protection with:

```bash
gh api repos/0xLizTech/stellarforge/branches/main/protection \
  --jq '.required_pull_request_reviews, .required_status_checks.contexts'
```

## Break-glass procedure

Emergency changes (for example, a live vulnerability) may bypass review **only**
when:

1. A maintainer announces the intent in the repository's internal channel first.
2. The bypass is limited to the smallest commit that fixes the incident.
3. A pull request is opened retroactively within 24 hours, linking the incident,
   and the usual checks run there.

If any of the three is skipped, the bypass is treated as a process incident and
reviewed at the next maintainers' sync.

## Related documents

- [`docs/MAINTAINERS.md`](MAINTAINERS.md) — review SLAs this policy enforces.
- [`docs/RELEASING.md`](RELEASING.md) — release workflow.
- [`docs/SECURITY.md`](SECURITY.md) — vulnerability handling.
