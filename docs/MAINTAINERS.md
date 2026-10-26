# Maintainers Guide

This guide is for people with write access to StellarForge. It documents how we
triage, review, and release so that expectations are the same for every
contributor and every maintainer.

## Who is a maintainer

[`.github/CODEOWNERS`](../.github/CODEOWNERS) is the source of truth for who
reviews changes. Maintainers are listed there and have write (or admin) access to
the repository.

**Becoming a maintainer.** Sustained, high-quality contributions — code, review,
triage, or documentation — are the qualification. Any existing maintainer may
nominate a contributor by opening a private discussion or issue; existing
maintainers decide by simple majority, and the nomination is recorded by adding
the person to CODEOWNERS.

**Emeritus.** A maintainer who has been inactive for six months or asks to step
back is moved to emeritus status in this document, keeping the record of their
contributions. Emeritus maintainers are welcome to return.

## Triage

- **New issues:** triage within **2 business days**. Apply at least one type
  label (`bug`, `enhancement`, `documentation`, `security`, `tooling`, `ci`), one
  stage label where applicable, and `good first issue` only when the task is
  genuinely self-contained with acceptance criteria.
- **Needs information:** ask and label `needs-info`. Close after two weeks of
  silence with a note that the issue can be reopened.
- **Duplicates:** close as duplicate of the canonical issue, linking both ways.
- **Security reports:** never triage in public. Follow
  [`docs/SECURITY.md`](SECURITY.md) and move the conversation to email.
- **Stale:** monthly, maintainers revisit issues labelled `needs-info`; anything
  else stays open until decided — silence is not a decision.

## Review

Review targets, measured from the last push:

| Stage | Target |
| --- | --- |
| First response (any maintainer) | 2 business days |
| Full review of a small PR (< 400 lines) | 5 business days |
| Full review of a large PR | 10 business days; ask for a split if it grows |

While reviewing:

- Check the PR template checklist is complete (`cargo fmt`, `clippy`, `test`,
  SDK `typecheck`/`test` where applicable).
- Verify the PR links its issue and the commit adds `Closes #N` when it closes
  one.
- Prefer correctness, tests, and security seams (authorization, overflow,
  storage/TTL, event shape) over style nits; CI enforces formatting.
- Ask for a guard test alongside every new public entry point, per the
  contributor checklist.
- Use the ADR test: does this change need an ADR or RFC? If yes, require it
  before merge.

**Merging.** A PR needs CI green and at least one maintainer approval, and
approvals must be re-requested after a substantive push (stale approvals are
dismissed in the branch protection settings). Do not merge your own PR without a
second maintainer's approval except for pure typo or dependency-bot fixes.

## Release duty

One maintainer is on **release duty** per release. The duty is:

1. Follow [`docs/RELEASING.md`](RELEASING.md) for the SDK and the contract wasm
   build (`.github/workflows/publish-sdk.yml`, `contracts-ci.yml`).
2. Curate `CHANGELOG.md`: every user-visible change under `Unreleased` gains an
   entry in the same PR that makes it, with the issue or PR number. Breaking
   changes get the `BREAKING` callout.
3. Rotate with the next release. Record the rotation in the release PR
   description.

## Decision records

- Protocol-level proposals go through the RFC process
  ([`docs/rfcs/README.md`](rfcs/README.md)).
- Architectural decisions accepted from RFCs (or made directly) become ADRs in
  [`docs/architecture/`](architecture/000-template.md) and are immutable once
  merged.
- See [`docs/BRANCH_PROTECTION.md`](BRANCH_PROTECTION.md) for the repository
  settings that enforce the review rules above.

## Escalation

- **Security:** `security@stellarforge.io`, per [`docs/SECURITY.md`](SECURITY.md).
- **Code of conduct:** `conduct@stellarforge.io`.
- **Disputed reviews:** a second maintainer makes the call; if opinions remain
  split, the RFC/ADR process is the tiebreaker.
