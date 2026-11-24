# StellarForge Open Source Issues

500 issues covering the development stages of the project, derived from [ROADMAP.md](ROADMAP.md), the [PRD](docs/prd/PRODUCT_REQUIREMENTS.md) and the [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md).

Publish them with `python scripts/issues/push_issues.py` (see `scripts/issues/README.md`).
Do not edit titles after publishing: the script uses them to skip issues that already exist.

## Stages

- Stage 1: Repository, Tooling and Developer Experience (26 issues)
- Stage 2: Phase 1 Hardening and Audit Follow-ups (52 issues)
- Stage 3: TypeScript SDK Maturity (54 issues)
- Stage 4: Phase 2 - Fractionalization Vault (41 issues)
- Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness (40 issues)
- Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution (63 issues)
- Stage 7: Phase 4 - Privacy and Cross-Chain (63 issues)
- Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting (54 issues)
- Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts) (44 issues)
- Stage 10: Research Tracks, Ecosystem and Community (33 issues)
- Stage 11: Operations, Observability and Release Engineering (30 issues)

---

<!-- Stage 1: Repository, Tooling and Developer Experience -->

## Add a devcontainer with Rust, stellar-cli and Node preinstalled

**Labels:** tooling, documentation, phase-1, good first issue

### Summary

Contributors must install Rust 1.98.1, the wasm target, stellar-cli and Node by hand. A devcontainer removes that setup cost.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a fresh Codespace runs `make test` and `npm test` without manual steps.

### Requirements

- [ ] Add `.devcontainer/devcontainer.json` pinned to `rust-toolchain.toml`
- [ ] Install stellar-cli at the version CI uses
- [ ] Run `npm ci` in `sdk/` on container creation

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A fresh Codespace runs `make test` and `npm test` without manual steps
- [ ] The CLI version matches `contracts-ci.yml`
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a VS Code tasks.json for build, test and lint

**Labels:** tooling, phase-1, good first issue

### Summary

The repo ships `.vscode/extensions.json` but no tasks. Contributors retype make targets.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: each task runs from the Command Palette.

### Requirements

- [ ] Add tasks for `make build`, `make test`, `make lint`, `npm test`
- [ ] Group them as build and test tasks
- [ ] Document the tasks in CONTRIBUTING.md

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Each task runs from the Command Palette
- [ ] CONTRIBUTING.md lists them
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add rust-analyzer settings for the workspace

**Labels:** tooling, phase-1, good first issue

### Summary

rust-analyzer may fail to enable the `testutils` feature, which hides test code from the IDE.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: opening `contracts/rwa-asset/tests` shows no unresolved imports.

### Requirements

- [ ] Add `.vscode/settings.json` enabling `testutils`
- [ ] Set the check command to clippy
- [ ] Confirm test files resolve symbols

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Opening `contracts/rwa-asset/tests` shows no unresolved imports
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add an .editorconfig for Rust, TypeScript, YAML and Markdown

**Labels:** tooling, phase-1, good first issue

### Summary

Mixed indentation and line endings appear in diffs from different editors.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: editors honoring EditorConfig produce no whitespace-only diffs.

### Requirements

- [ ] Set indent style and size per file type
- [ ] Set `end_of_line = lf`
- [ ] Trim trailing whitespace except in Markdown

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Editors honoring EditorConfig produce no whitespace-only diffs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add .gitattributes to normalize line endings

**Labels:** tooling, phase-1, good first issue

### Summary

Windows contributors commit CRLF in scripts, which breaks `scripts/deploy.sh` on Linux CI.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: `git ls-files --eol` shows no CRLF in text files.

### Requirements

- [ ] Force LF for `*.sh`, `*.rs`, `*.ts`, `*.md`
- [ ] Mark `Cargo.lock` and `package-lock.json` as generated
- [ ] Re-normalize the tree once

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] `git ls-files --eol` shows no CRLF in text files
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a make doctor target that checks the toolchain

**Labels:** tooling, phase-1, good first issue

### Summary

New contributors cannot tell which tool is missing when a build fails.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: `make doctor` reports all missing tools in one run.

### Requirements

- [ ] Check rustc, wasm target, stellar-cli, node and npm versions
- [ ] Print the expected version beside each result
- [ ] Exit non-zero if anything is missing

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] `make doctor` reports all missing tools in one run
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a make help target that lists every target

**Labels:** tooling, phase-1, good first issue

### Summary

The Makefile declares `help` in `.PHONY` but the output is minimal.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: `make help` prints every target with a one-line description.

### Requirements

- [ ] Add `##` descriptions to every target
- [ ] Generate help output with awk
- [ ] Group targets by contracts, sdk and release

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] `make help` prints every target with a one-line description
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add make targets for SDK build, test and lint

**Labels:** tooling, phase-1, good first issue

### Summary

SDK commands live only in `sdk/package.json`, so contributors run two different tool families.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: `make all` fails if the SDK typecheck fails.

### Requirements

- [ ] Add `sdk-build`, `sdk-test`, `sdk-lint`, `sdk-typecheck`
- [ ] Make `make all` include SDK checks
- [ ] Update CONTRIBUTING.md

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] `make all` fails if the SDK typecheck fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add pre-commit hooks for rustfmt and eslint

**Labels:** tooling, phase-1, difficulty: medium

### Summary

Formatting failures reach CI because nothing runs locally.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a commit with bad formatting is rejected locally.

### Requirements

- [ ] Add a `.pre-commit-config.yaml`
- [ ] Run `cargo fmt --check` and eslint on staged files
- [ ] Document install steps

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A commit with bad formatting is rejected locally
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add commitlint for conventional commits

**Labels:** tooling, ci, phase-1, difficulty: medium

### Summary

CONTRIBUTING.md defines a commit style but nothing enforces it.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a PR with a non-conforming commit fails the check.

### Requirements

- [ ] Add a commitlint config matching the documented types
- [ ] Add a CI job that lints PR commits
- [ ] Document allowed scopes

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A PR with a non-conforming commit fails the check
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- `.github/workflows/`

---

## Add cargo-deny configuration for licenses and bans

**Labels:** security, ci, phase-1, difficulty: medium

### Summary

Dependencies are audited for advisories but not for license compatibility.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: cI fails when a GPL dependency is added.

### Requirements

- [ ] Add `deny.toml` allowing Apache-2.0, MIT, BSD
- [ ] Deny unknown registries
- [ ] Run `cargo deny check` in `security.yml`

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] CI fails when a GPL dependency is added
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add cargo-udeps or cargo-machete to find unused dependencies

**Labels:** tooling, ci, phase-1, good first issue

### Summary

Unused crates increase WASM size and audit surface.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: the job passes on main and fails when an unused dependency is added.

### Requirements

- [ ] Run the tool across the workspace
- [ ] Remove unused dependencies it finds
- [ ] Add a scheduled CI job

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] The job passes on main and fails when an unused dependency is added
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- `.github/workflows/`

---

## Track WASM size per contract in CI

**Labels:** contracts, ci, phase-1, difficulty: medium

### Summary

Soroban has a contract size limit. A silent size regression can block deployment.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a PR that adds 10 KB to a contract shows the delta.

### Requirements

- [ ] Build each contract and record the byte size
- [ ] Comment the size delta on pull requests
- [ ] Fail above a configured budget

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A PR that adds 10 KB to a contract shows the delta
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add a size budget file for each contract

**Labels:** contracts, ci, phase-1, good first issue

### Summary

A budget needs a written number to compare against.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: raising a budget requires editing a tracked file.

### Requirements

- [ ] Add `contracts/size-budget.json`
- [ ] Set a budget for each of the five contracts
- [ ] Read it from the size CI job

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Raising a budget requires editing a tracked file
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add soroban-sdk version consistency check

**Labels:** contracts, ci, phase-1, good first issue

### Summary

Workspace members could drift to different `soroban-sdk` versions.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a crate pinning its own `soroban-sdk` fails CI.

### Requirements

- [ ] Use `[workspace.dependencies]` for the SDK
- [ ] Add a CI script that rejects per-crate overrides
- [ ] Document the upgrade procedure

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A crate pinning its own `soroban-sdk` fails CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add a dependabot group for Soroban crates

**Labels:** ci, phase-1, good first issue

### Summary

Dependabot opens one PR per crate, which breaks lockstep upgrades.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: one PR upgrades all Soroban crates together.

### Requirements

- [ ] Group `soroban-*` crates in `dependabot.yml`
- [ ] Group SDK dev dependencies
- [ ] Set weekly cadence

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] One PR upgrades all Soroban crates together
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Add a CODEOWNERS entry per area

**Labels:** ci, phase-1, good first issue

### Summary

Review routing is coarse. Contracts and SDK need different reviewers.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: pRs touching workflows request the security reviewer.

### Requirements

- [ ] Map `contracts/` and `sdk/` and `docs/` to teams
- [ ] Require review for `.github/workflows/`
- [ ] Document the policy

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] PRs touching workflows request the security reviewer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Add a labels.yml and a sync workflow

**Labels:** ci, tooling, phase-1, difficulty: medium

### Summary

Issue labels are created by hand and drift.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: running the workflow creates every label with a color.

### Requirements

- [ ] Define area, phase, difficulty and type labels
- [ ] Add a workflow that syncs labels
- [ ] Document the taxonomy in CONTRIBUTING.md

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Running the workflow creates every label with a color
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- `scripts/`
- [Makefile](Makefile)

---

## Add issue templates for security hardening and tech debt

**Labels:** documentation, phase-1, good first issue

### Summary

Only bug and feature templates exist. Hardening work has no home.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: new templates appear in the issue chooser.

### Requirements

- [ ] Add two templates under `.github/ISSUE_TEMPLATE`
- [ ] Include a threat description field
- [ ] Link from `config.yml`

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] New templates appear in the issue chooser
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a pull request size labeler

**Labels:** ci, phase-1, good first issue

### Summary

Large PRs are hard to review and nothing flags them.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: every PR receives a size label.

### Requirements

- [ ] Add a labeler workflow for size/S, M, L, XL
- [ ] Warn on XL
- [ ] Document the guideline

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Every PR receives a size label
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Add a stale issue policy

**Labels:** ci, phase-1, good first issue

### Summary

Old issues without activity accumulate.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: inactive issues are marked and closed on schedule.

### Requirements

- [ ] Add `stale.yml` with a 60 day warning
- [ ] Exempt `security` and `roadmap` labels
- [ ] Close after 14 more days

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] Inactive issues are marked and closed on schedule
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Add a CONTRIBUTING section for running the testnet smoke tests

**Labels:** documentation, phase-1, good first issue

### Summary

`smoke.testnet.test.ts` needs funded accounts but the steps are not written.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a new contributor runs the smoke test successfully.

### Requirements

- [ ] Document friendbot funding
- [ ] Document required environment variables
- [ ] Warn about rate limits

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A new contributor runs the smoke test successfully
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a glossary document for RWA and Soroban terms

**Labels:** documentation, phase-1, good first issue

### Summary

Docs use terms such as TTL, stroops, SEP-40 and tranche without definitions.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: rEADME links to the glossary.

### Requirements

- [ ] Create `docs/GLOSSARY.md`
- [ ] Define at least 40 terms
- [ ] Link it from the README

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] README links to the glossary
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a FAQ document for integrators

**Labels:** documentation, phase-1, good first issue

### Summary

The same integration questions repeat in discussions.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: fAQ answers at least 15 questions.

### Requirements

- [ ] Create `docs/FAQ.md`
- [ ] Cover keys, fees, TTL and upgrades
- [ ] Link it from README

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] FAQ answers at least 15 questions
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a repository-wide link checker to CI

**Labels:** ci, documentation, phase-1, difficulty: medium

### Summary

Documentation links rot after file moves.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a broken relative link fails the job.

### Requirements

- [ ] Run lychee or markdown-link-check on `*.md`
- [ ] Cache results
- [ ] Allow a configured ignore list

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A broken relative link fails the job
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a spell check for documentation

**Labels:** ci, documentation, phase-1, good first issue

### Summary

Typos appear in ADRs and the PRD.

### Context

StellarForge is a Soroban workspace (`contracts/*`) plus a TypeScript SDK (`sdk/`). Phase 1 is feature complete, so this stage removes friction for new contributors and makes the toolchain trustworthy before more contracts land.

This issue belongs to **Stage 1: Repository, Tooling and Developer Experience**.

### Objective

Deliver the change described above so that: a misspelled word in docs fails the job.

### Requirements

- [ ] Add a cspell config with a project dictionary
- [ ] Run it on `docs/` and README in CI
- [ ] List Soroban terms in the dictionary

### Out of Scope

Changes to contract behavior or SDK public APIs unless the issue says so.

### Acceptance Criteria

- [ ] A misspelled word in docs fails the job
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 2: Phase 1 Hardening and Audit Follow-ups -->

## Demonstrate byte-for-byte reproducible WASM on a second machine

**Labels:** security, ci, phase-1, difficulty: hard, help wanted

### Summary

IR-10 pinned the toolchain, but the audit states reproduction on a second machine was never shown.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: hashes match across machines.

### Requirements

- [ ] Build all contracts on two clean environments
- [ ] Compare SHA-256 sums with the `build-wasm` artifact
- [ ] Record the procedure in `docs/RELEASING.md`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Hashes match across machines
- [ ] Any mismatch has a documented cause
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add a CI job that rebuilds WASM twice and diffs hashes

**Labels:** ci, security, phase-1, difficulty: medium

### Summary

Reproducibility should be checked on every PR instead of once.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: the job fails when a build step introduces nondeterminism.

### Requirements

- [ ] Build twice in separate runners
- [ ] Fail on any hash difference
- [ ] Upload both hash lists

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] The job fails when a build step introduces nondeterminism
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Configure required reviewers for the npm-publish environment

**Labels:** ci, security, phase-1, good first issue

### Summary

IR-12 notes the environment exists but reviewers are set only in repository settings.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a publish run pauses for approval.

### Requirements

- [ ] Add the reviewers in GitHub settings
- [ ] Document the setup in `docs/RELEASING.md`
- [ ] Add a checklist item to the release template

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A publish run pauses for approval
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Run the hardened security.yml on GitHub and record the result

**Labels:** ci, security, phase-1, good first issue

### Summary

IR-11 states the workflow changes have not yet run on GitHub.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: the workflow is green on main.

### Requirements

- [ ] Trigger the workflow on a PR
- [ ] Fix any failure
- [ ] Record the run URL in the audit doc

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] The workflow is green on main
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add npm audit signatures verification for SDK releases

**Labels:** ci, security, phase-1, difficulty: medium

### Summary

Published SDK tarballs should carry provenance.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: the npm page shows a provenance badge.

### Requirements

- [ ] Publish with `--provenance`
- [ ] Verify with `npm audit signatures` in CI
- [ ] Document verification for consumers

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] The npm page shows a provenance badge
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Pin every GitHub Action to a commit SHA

**Labels:** ci, security, phase-1, good first issue

### Summary

IR-12 pinned some actions. Remaining workflows may still use mutable tags.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: no workflow references a mutable tag.

### Requirements

- [ ] List every `uses:` entry
- [ ] Replace tags with full SHAs and a version comment
- [ ] Let Dependabot update them

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] No workflow references a mutable tag
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add zizmor or actionlint to lint workflows

**Labels:** ci, security, phase-1, difficulty: medium

### Summary

Workflow files can contain injection patterns that review misses.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: workflow lint runs on every PR touching `.github/`.

### Requirements

- [ ] Add actionlint to CI
- [ ] Fix findings
- [ ] Add zizmor for security rules

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Workflow lint runs on every PR touching `.github/`
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add SECURITY.md disclosure policy to the repository root

**Labels:** documentation, security, phase-1, good first issue

### Summary

The policy lives at `docs/SECURITY.md`, but GitHub looks for a root or `.github` copy.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: the Security tab shows the policy.

### Requirements

- [ ] Add `.github/SECURITY.md` linking to the docs file
- [ ] Enable private vulnerability reporting
- [ ] Document response times

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] The Security tab shows the policy
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Fuzz rwa-asset transfer and allowance paths

**Labels:** contracts, testing, security, phase-1, difficulty: hard, help wanted

### Summary

Property bugs in balance math are costly in a tokenization contract.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a seeded bug in `transfer` is found in under 5 minutes.

### Requirements

- [ ] Add `cargo-fuzz` or proptest targets
- [ ] Assert supply equals the sum of balances
- [ ] Run for a fixed budget in nightly CI

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A seeded bug in `transfer` is found in under 5 minutes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add property tests for the mint and burn supply invariant

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

Total supply must always equal minted minus burned.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: 10,000 random sequences keep the invariant.

### Requirements

- [ ] Generate random operation sequences with proptest
- [ ] Check the invariant after each step
- [ ] Include the supply cap

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] 10,000 random sequences keep the invariant
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add property tests for allowance expiry

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

IR-09 added `live_until_ledger`. Boundary ledgers deserve randomized tests.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: no sequence spends an expired allowance.

### Requirements

- [ ] Test ledgers at expiry minus one, equal, plus one
- [ ] Test overwriting and zeroing
- [ ] Test `burn_from` with expired allowance

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] No sequence spends an expired allowance
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add a test that every state-changing entry point emits an event

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

IR-03 added events, and nothing prevents a new function from skipping them.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: adding a silent mutator fails the test.

### Requirements

- [ ] Enumerate entry points through the generated spec
- [ ] Assert each mutating call publishes an event
- [ ] Fail on a new function without a mapping

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Adding a silent mutator fails the test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Pin event topic and data layout in snapshot tests

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

Indexers depend on event shapes. Silent changes break them.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: changing an event field fails a test.

### Requirements

- [ ] Snapshot every event for all five contracts
- [ ] Store snapshots in the repo
- [ ] Document how to update them

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Changing an event field fails a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add storage TTL tests for every persistent key

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

IR-07 fixed TTL gaps in two functions. The same gap may exist elsewhere.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a key without TTL extension fails a test.

### Requirements

- [ ] List every persistent and instance key
- [ ] Assert TTL is extended on write and on read where required
- [ ] Add a test for each key family

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A key without TTL extension fails a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Audit contracts for unbounded vectors in storage

**Labels:** contracts, security, phase-1, difficulty: medium

### Summary

Unbounded `Vec` values grow until a transaction fails (IR-01 was one case).

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a written inventory exists with a verdict per item.

### Requirements

- [ ] Grep every `Vec` in stored types
- [ ] Classify each as bounded or unbounded
- [ ] Open a fix for each unbounded case

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A written inventory exists with a verdict per item
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add resource-limit tests under mainnet budget settings

**Labels:** contracts, testing, phase-1, difficulty: hard, help wanted

### Summary

Tests default to generous budgets and miss CPU or memory failures.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: instruction counts appear in the CI log.

### Requirements

- [ ] Run key flows with mainnet limits
- [ ] Record instruction counts
- [ ] Fail on a configured percentage regression

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Instruction counts appear in the CI log
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Record a gas and resource baseline for every entry point

**Labels:** contracts, testing, phase-1, difficulty: medium

### Summary

Resource cost has no tracked baseline, so regressions pass unnoticed.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: baseline file exists and is documented.

### Requirements

- [ ] Measure CPU, memory and ledger reads and writes per function
- [ ] Store results in `docs/benchmarks/`
- [ ] Add a script to regenerate them

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Baseline file exists and is documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Review admin rotation for lost-key recovery

**Labels:** contracts, security, phase-1, difficulty: hard, help wanted

### Summary

IR-04 gave dual-authorization rotation but a lost admin key has no recovery.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: aDR-006 is merged with a decision.

### Requirements

- [ ] Write an ADR comparing timelock, multisig and recovery guardian
- [ ] Choose an approach
- [ ] Open implementation issues

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] ADR-006 is merged with a decision
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add a two-step admin transfer with acceptance

**Labels:** contracts, security, phase-1, difficulty: medium

### Summary

A typo in `new_admin` locks the contract if acceptance is not required.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a transfer to an uncontrolled address is impossible.

### Requirements

- [ ] Add `propose_admin` and `accept_admin` or confirm current dual auth is sufficient
- [ ] Add tests for wrong address
- [ ] Update SDK clients

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A transfer to an uncontrolled address is impossible
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add an emergency pause to compliance, registry and oracle-adapter

**Labels:** contracts, security, phase-1, difficulty: medium

### Summary

Only `rwa-asset` can pause. A compromised compliance admin has no circuit breaker.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: paused contracts reject all state changes.

### Requirements

- [ ] Add `set_paused` and checks to the three contracts
- [ ] Emit `Paused` events
- [ ] Add tests per contract

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Paused contracts reject all state changes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Document the trust assumptions of every privileged role

**Labels:** documentation, security, phase-1, good first issue

### Summary

Roles such as issuer, reporter and admin have scattered descriptions.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: matrix covers all five contracts.

### Requirements

- [ ] Create a role matrix in `docs/SECURITY.md`
- [ ] List the powers and limits of each role
- [ ] Link tests that prove the limits

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Matrix covers all five contracts
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write a threat model for rwa-asset

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

The PRD lists threats at protocol level only.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: each entry point has at least one analyzed threat.

### Requirements

- [ ] Use STRIDE per entry point
- [ ] Record mitigations and gaps
- [ ] Store in `docs/threat-models/rwa-asset.md`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Each entry point has at least one analyzed threat
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write a threat model for compliance

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

Compliance data decides who may transfer. It needs its own analysis.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: document merged and linked from the audit handover.

### Requirements

- [ ] Cover stale records, admin compromise and jurisdiction spoofing
- [ ] Record gaps
- [ ] Store in `docs/threat-models/`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Document merged and linked from the audit handover
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write a threat model for registry

**Labels:** documentation, security, phase-1, good first issue

### Summary

IR-16 documented that registry entries are assertions. A threat model should make this explicit.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: document merged.

### Requirements

- [ ] Cover fake entries, spam and delisting
- [ ] Describe indexer guidance
- [ ] Store in `docs/threat-models/`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Document merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write a threat model for governance

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

Voting weight is supplied by the caller in `vote`. This is a central risk.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: document merged.

### Requirements

- [ ] Cover weight manipulation, short voting periods and admin power
- [ ] Record planned mitigations
- [ ] Store in `docs/threat-models/`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Document merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write a threat model for oracle-adapter

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

Price feeds are a classic attack surface.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: document merged.

### Requirements

- [ ] Cover reporter compromise, stale prices and deviation bypass
- [ ] Link ADR-005
- [ ] Store in `docs/threat-models/`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Document merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Restrict governance vote weight to a verified source

**Labels:** contracts, security, phase-1, difficulty: hard, help wanted

### Summary

`vote` accepts `weight: i128` from the voter. Any caller can claim any weight.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a voter cannot exceed the weight provided by the source.

### Requirements

- [ ] Define a weight source (token balance or snapshot)
- [ ] Reject caller-supplied weight
- [ ] Add tests with forged weight

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A voter cannot exceed the weight provided by the source
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add a proposal deposit option to governance

**Labels:** contracts, phase-1, difficulty: medium

### Summary

IR-14 left proposal spam unmitigated apart from input bounds.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: spam proposals cost the proposer.

### Requirements

- [ ] Add an optional deposit configured by admin
- [ ] Refund on finalization
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Spam proposals cost the proposer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add quorum and threshold parameters to governance

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Finalization today needs richer rules than simple majority.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a proposal below quorum fails.

### Requirements

- [ ] Add quorum and pass threshold to the constructor or setters
- [ ] Update `finalize`
- [ ] Add tests for edge cases

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A proposal below quorum fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Reject duplicate proposals with the same metadata hash

**Labels:** contracts, phase-1, good first issue

### Summary

Identical proposals can flood the voting list.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: second identical proposal returns an error.

### Requirements

- [ ] Store the hash of the title and action
- [ ] Reject a duplicate while active
- [ ] Add a test

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Second identical proposal returns an error
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a cancel_proposal entry point to governance

**Labels:** contracts, phase-1, difficulty: medium

### Summary

A mistaken proposal cannot be withdrawn.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: cancelled proposals reject votes.

### Requirements

- [ ] Allow the proposer or admin to cancel before the first vote
- [ ] Emit `ProposalCancelled`
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Cancelled proposals reject votes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a pagination API for governance proposals

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Only `proposal_count` and `get_proposal` exist. Frontends cannot list proposals efficiently.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: paging returns stable ordering.

### Requirements

- [ ] Add `list_proposals(start, limit)` capped at 100
- [ ] Reuse the registry pagination pattern
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Paging returns stable ordering
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Enforce a compliance level upper bound consistently

**Labels:** contracts, phase-1, good first issue

### Summary

`MAX_LEVEL` exists in compliance but the asset contract takes any `min_level`.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: `set_compliance` with level 9 fails.

### Requirements

- [ ] Reject `min_level` above `MAX_LEVEL` in `set_compliance`
- [ ] Add a test
- [ ] Update docs

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] `set_compliance` with level 9 fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add batch KYC set and revoke to compliance

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Onboarding many holders requires one transaction each.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: one call stores 50 records atomically.

### Requirements

- [ ] Add `set_kyc_batch` capped at 50 records
- [ ] Emit one event per record
- [ ] Add tests for partial failure

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] One call stores 50 records atomically
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add jurisdiction allow and deny lists to rwa-asset

**Labels:** contracts, phase-1, difficulty: hard, help wanted

### Summary

Issuers must block entire jurisdictions, not only levels.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a holder in a denied jurisdiction cannot receive tokens.

### Requirements

- [ ] Store an allow or deny list of ISO codes
- [ ] Check it on transfer
- [ ] Add tests with several jurisdictions

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A holder in a denied jurisdiction cannot receive tokens
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add transfer restriction events for blocked transfers

**Labels:** contracts, phase-1, good first issue

### Summary

Rejected transfers revert, so indexers cannot see what was attempted.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: aDR note merged.

### Requirements

- [ ] Evaluate emitting a diagnostic event only in simulation
- [ ] Document the limitation
- [ ] Decide in an ADR

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] ADR note merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Handle compliance contract upgrade or removal safely

**Labels:** contracts, security, phase-1, difficulty: medium

### Summary

Setting `compliance` to `None` or to a broken contract can freeze or open transfers.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: a failing compliance call has defined behavior.

### Requirements

- [ ] Define behavior when the compliance call fails
- [ ] Add tests with a trap contract
- [ ] Document the rule

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] A failing compliance call has defined behavior
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add a forced transfer function for legal orders

**Labels:** contracts, phase-1, difficulty: hard, help wanted

### Summary

Regulated issuers need court-ordered recovery.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: only the role can call it.

### Requirements

- [ ] Add `forced_transfer` callable by a dedicated role
- [ ] Emit a distinct event with a reason hash
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Only the role can call it
- [ ] every call is logged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a frozen-address list to rwa-asset

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Issuers must freeze a single holder without pausing everyone.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: frozen holder cannot send or receive.

### Requirements

- [ ] Add `freeze` and `unfreeze`
- [ ] Block transfers from and to frozen addresses
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Frozen holder cannot send or receive
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add holder count tracking to rwa-asset

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Reg D style limits depend on the number of holders.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: holder count matches balances in a property test.

### Requirements

- [ ] Track holders with a counter
- [ ] Update on first receive and zero balance
- [ ] Add an optional holder cap

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Holder count matches balances in a property test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a lock-up date per holder to rwa-asset

**Labels:** contracts, phase-1, difficulty: medium

### Summary

Some securities restrict resale for a period.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: transfer before unlock fails.

### Requirements

- [ ] Store an unlock timestamp per holder
- [ ] Reject transfers before it
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Transfer before unlock fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Make supply cap changes governance-controlled

**Labels:** contracts, phase-1, difficulty: medium

### Summary

The admin can tighten a cap alone. Loosening needs wider approval.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: behavior matches the documented rule.

### Requirements

- [ ] Define who may change the cap
- [ ] Add a test for each direction
- [ ] Document in the ADR

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Behavior matches the documented rule
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a metadata version field and history events

**Labels:** contracts, phase-1, good first issue

### Summary

`update_metadata` overwrites state. Integrators need a change history.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: every update increments the version.

### Requirements

- [ ] Add a version counter
- [ ] Emit old and new hashes
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Every update increments the version
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Validate legal document hash format on-chain

**Labels:** contracts, phase-1, good first issue

### Summary

A zero or wrong-length hash can be stored as legal anchor.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: zero hash is rejected at construction and update.

### Requirements

- [ ] Reject an all-zero hash
- [ ] Document the expected algorithm
- [ ] Add tests

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Zero hash is rejected at construction and update
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a contract-level README for each crate

**Labels:** documentation, phase-1, good first issue

### Summary

Crates have no local documentation.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: each crate has a README.

### Requirements

- [ ] Add `README.md` to each contract folder
- [ ] Describe functions, errors and events
- [ ] Link from the root README

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Each crate has a README
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Generate error code tables for each contract

**Labels:** documentation, phase-1, good first issue

### Summary

Integrators need a table of error codes and meanings.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: table matches the source.

### Requirements

- [ ] Generate a Markdown table from `error.rs`
- [ ] Add it to each crate README
- [ ] Add a CI check for drift

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Table matches the source
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Generate event reference tables for each contract

**Labels:** documentation, phase-1, good first issue

### Summary

Event layouts are only in code.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: table matches the source.

### Requirements

- [ ] Generate a Markdown event table
- [ ] Include topics and data types
- [ ] Add a CI drift check

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Table matches the source
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a CHANGELOG entry template and enforce it

**Labels:** documentation, ci, phase-1, good first issue

### Summary

CHANGELOG.md updates are inconsistent.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: pRs with code changes require a changelog line.

### Requirements

- [ ] Define sections per release
- [ ] Add a PR checklist item
- [ ] Add a CI check for unreleased notes

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] PRs with code changes require a changelog line
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `.github/workflows/`

---

## Complete the external audit handover package

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

`external-audit-handover-phase1.md` exists. It needs a final completeness pass.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: handover reviewed by two maintainers.

### Requirements

- [ ] Check scope, build instructions and known issues
- [ ] Add commit hashes
- [ ] Add contact points

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Handover reviewed by two maintainers
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Create an audit findings tracking template

**Labels:** documentation, security, phase-1, good first issue

### Summary

External findings need a standard tracking format.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: template merged.

### Requirements

- [ ] Add `docs/audits/TEMPLATE-finding.md`
- [ ] Include severity, status and fix commit
- [ ] Link from the handover

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Template merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add a bug bounty program draft

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

A bounty needs rules before mainnet.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: draft reviewed and merged.

### Requirements

- [ ] Draft scope, rewards and safe harbor
- [ ] Review with legal
- [ ] Store in `docs/BUG_BOUNTY.md`

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Draft reviewed and merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add an incident response runbook

**Labels:** documentation, security, phase-1, difficulty: medium

### Summary

PRD 10.4 mentions emergency response, but no runbook exists.

### Context

The internal review in `docs/audits/2026-09-14-internal-review-phase1.md` fixed IR-01 to IR-17 in code, but several fixes carry open follow-ups (unverified claims, repository settings, missing tests). This stage closes them before the external audit.

This issue belongs to **Stage 2: Phase 1 Hardening and Audit Follow-ups**.

### Objective

Deliver the change described above so that: runbook merged and drilled once on testnet.

### Requirements

- [ ] Write steps for pause, communication and postmortem
- [ ] Name roles
- [ ] Add a drill checklist

### Out of Scope

New protocol features. Every item here either proves an existing fix or removes a known residual risk.

### Acceptance Criteria

- [ ] Runbook merged and drilled once on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

<!-- Stage 3: TypeScript SDK Maturity -->

## Add a typed error mapping for every contract error code

**Labels:** sdk, phase-2, difficulty: medium

### Summary

`errors.ts` exists but contract error codes may surface as raw numbers.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: catching `RwaAssetError` exposes a readable code name.

### Requirements

- [ ] Map each contract error enum to a named SDK error class
- [ ] Preserve the numeric code
- [ ] Add tests per contract

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Catching `RwaAssetError` exposes a readable code name
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Generate contract bindings from the WASM spec in CI

**Labels:** sdk, ci, phase-2, difficulty: hard, help wanted

### Summary

Hand-written clients drift from the contract ABI.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: changing a contract function fails SDK CI until types update.

### Requirements

- [ ] Run `stellar contract bindings typescript` in CI
- [ ] Diff against checked-in types
- [ ] Fail on drift

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Changing a contract function fails SDK CI until types update
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`

---

## Add a client factory that reads a deployment manifest

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Integrators pass contract IDs by hand.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: one call returns all five clients.

### Requirements

- [ ] Define a `deployments.json` schema
- [ ] Add `StellarForge.fromManifest(network)`
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] One call returns all five clients
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Publish a deployments manifest for testnet

**Labels:** sdk, tooling, phase-2, good first issue

### Summary

The factory needs a real manifest.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: smoke test reads the manifest.

### Requirements

- [ ] Write IDs after each testnet deploy
- [ ] Commit `deployments/testnet.json`
- [ ] Verify IDs resolve in the smoke test

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Smoke test reads the manifest
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add SDK support for Soroban RPC retry with backoff

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Transient RPC failures abort writes.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: a 503 followed by 200 succeeds.

### Requirements

- [ ] Add configurable retry with jitter
- [ ] Retry only idempotent reads and safe states
- [ ] Add tests with a mock RPC

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] A 503 followed by 200 succeeds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add timeouts and AbortSignal support to all calls

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Long calls cannot be cancelled.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: aborting stops polling within one interval.

### Requirements

- [ ] Accept `AbortSignal` in options
- [ ] Abort polling loops cleanly
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Aborting stops polling within one interval
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a pluggable signer interface

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Signing is tied to a keypair. Wallets and HSMs need an adapter.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: a custom signer works for any write.

### Requirements

- [ ] Define `Signer { publicKey, signTransaction }`
- [ ] Implement Keypair and Freighter-style adapters
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] A custom signer works for any write
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a Freighter wallet signer example

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Browser dApps need a ready adapter.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: example runs in a sample page.

### Requirements

- [ ] Add an example using the Freighter API
- [ ] Document network passphrase handling
- [ ] Add to sdk/README

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Example runs in a sample page
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a Ledger hardware wallet signer example

**Labels:** sdk, documentation, phase-2, difficulty: medium

### Summary

Institutional issuers use hardware signing.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: example signs a mint on testnet.

### Requirements

- [ ] Write the example with Stellar Ledger app
- [ ] Document limits
- [ ] Add to sdk/README

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Example signs a mint on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add multi-signature transaction assembly helpers

**Labels:** sdk, phase-2, difficulty: hard, help wanted

### Summary

Admin actions may require several signers.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: two signers sign one XDR in sequence.

### Requirements

- [ ] Add helpers to collect signatures on one XDR
- [ ] Validate threshold
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Two signers sign one XDR in sequence
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a simulate-only dry-run mode to every write

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Users want a fee and result preview before signing.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: no submission happens in dry-run.

### Requirements

- [ ] Add `{ simulateOnly: true }`
- [ ] Return resource fee and return value
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] No submission happens in dry-run
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Expose resource fee estimates with a safety margin option

**Labels:** sdk, phase-2, good first issue

### Summary

Fixed fees fail on busy ledgers.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: fee scales as configured.

### Requirements

- [ ] Add `feeMultiplier` option
- [ ] Default to a documented value
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Fee scales as configured
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add event parsing helpers for all contract events

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Consumers decode events by hand.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: decoding a real testnet event yields typed data.

### Requirements

- [ ] Add typed decoders for each event
- [ ] Export a union type
- [ ] Add fixtures

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Decoding a real testnet event yields typed data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add an event streaming helper with cursor persistence

**Labels:** sdk, phase-2, difficulty: hard, help wanted

### Summary

Indexers need resume-after-restart.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: restarting resumes without duplicates.

### Requirements

- [ ] Poll `getEvents` with a cursor
- [ ] Accept a storage adapter
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Restarting resumes without duplicates
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a getEvents pagination helper

**Labels:** sdk, phase-2, good first issue

### Summary

RPC event queries are paged.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: a 1,000-event range returns all events.

### Requirements

- [ ] Loop until the cursor ends
- [ ] Cap total results
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] A 1,000-event range returns all events
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add typed return values for registry list pagination

**Labels:** sdk, phase-2, good first issue

### Summary

Callers page manually.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: `for await` walks 250 assets.

### Requirements

- [ ] Add an async iterator over `list_assets`
- [ ] Respect the 100 page cap
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] `for await` walks 250 assets
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add an iterator over governance proposals

**Labels:** sdk, phase-2, good first issue

### Summary

There is no easy listing.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: iterator yields all proposals.

### Requirements

- [ ] Add `proposals()` iterator using `proposal_count`
- [ ] Handle missing entries
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Iterator yields all proposals
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add bigint-safe amount helpers with decimals awareness

**Labels:** sdk, phase-2, difficulty: medium

### Summary

`toStroops` and `fromStroops` assume 7 decimals. Assets can differ.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: round trip is exact for any decimals.

### Requirements

- [ ] Add decimals parameter
- [ ] Reject excess precision
- [ ] Add tests for 0 to 18 decimals

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Round trip is exact for any decimals
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add display formatting helpers for amounts

**Labels:** sdk, phase-2, good first issue

### Summary

UIs need grouped, locale-aware strings.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: output is exact for 1e30 values.

### Requirements

- [ ] Add `formatAmount(amount, decimals, locale)`
- [ ] Avoid floating point
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Output is exact for 1e30 values
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Validate addresses in every client method

**Labels:** sdk, phase-2, good first issue

### Summary

Bad addresses fail deep inside the XDR layer.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: error message names the bad parameter.

### Requirements

- [ ] Validate G, C and M addresses at call entry
- [ ] Throw `InvalidAddressError`
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Error message names the bad parameter
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add MuxedAddress support in transfer helpers

**Labels:** sdk, phase-2, difficulty: medium

### Summary

The contract accepts `MuxedAddress` but the SDK may not expose it.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: transfer to an M address succeeds.

### Requirements

- [ ] Accept M addresses in `transfer`
- [ ] Parse the id
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Transfer to an M address succeeds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add SEP-41 compatibility test against the Stellar token interface

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

RWA tokens should work in wallets that speak SEP-41.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: compatibility table is documented.

### Requirements

- [ ] Run a generic SEP-41 client against the contract on testnet
- [ ] List unsupported methods
- [ ] Fix gaps

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Compatibility table is documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add ESM and CJS import tests to the package

**Labels:** sdk, ci, phase-2, good first issue

### Summary

Dual builds can break one format.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: both imports pass in CI.

### Requirements

- [ ] Add a CI job that imports both formats from the packed tarball
- [ ] Check type resolution
- [ ] Add to publish flow

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Both imports pass in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`

---

## Add bundle size tracking for the SDK

**Labels:** sdk, ci, phase-2, good first issue

### Summary

Large bundles hurt browser dApps.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: budget is enforced.

### Requirements

- [ ] Measure with size-limit
- [ ] Set a budget
- [ ] Fail on regression

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Budget is enforced
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`

---

## Make the SDK tree-shakeable

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Importing one client pulls in all.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: importing only RwaAsset omits governance code.

### Requirements

- [ ] Set `sideEffects: false`
- [ ] Split exports by client
- [ ] Verify with a bundler test

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Importing only RwaAsset omits governance code
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add browser compatibility tests

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

Node-only APIs may slip in.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: browser suite is green.

### Requirements

- [ ] Run tests in a browser runner
- [ ] Polyfill nothing silently
- [ ] Document supported browsers

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Browser suite is green
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add Deno and Bun smoke tests

**Labels:** sdk, testing, phase-2, good first issue

### Summary

Users run other runtimes.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: both runtimes pass.

### Requirements

- [ ] Import the package in Deno and Bun in CI
- [ ] Run a read call against testnet mock
- [ ] Document support

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Both runtimes pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Generate API reference docs with TypeDoc

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Documentation of public classes is only in source.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: docs site builds in CI.

### Requirements

- [ ] Add TypeDoc
- [ ] Publish to GitHub Pages
- [ ] Link from README

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Docs site builds in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add TSDoc comments to all public exports

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Many exports lack doc comments.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: lint fails on undocumented export.

### Requirements

- [ ] Document parameters, returns and errors
- [ ] Add examples to key methods
- [ ] Enforce with eslint plugin

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Lint fails on undocumented export
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add runnable examples folder for the SDK

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

New users need copy-paste flows.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: each example runs end to end.

### Requirements

- [ ] Add examples for mint, transfer, KYC, propose, vote
- [ ] Make them run on testnet
- [ ] Test them in CI

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Each example runs end to end
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a CLI wrapper around the SDK

**Labels:** sdk, tooling, phase-2, difficulty: hard, help wanted

### Summary

Operators prefer a command line to scripts.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: `sforge asset mint` works on testnet.

### Requirements

- [ ] Add a `sforge` binary with subcommands per contract
- [ ] Read config from env
- [ ] Add help text

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] `sforge asset mint` works on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add JSON output mode to the CLI

**Labels:** sdk, tooling, phase-2, good first issue

### Summary

Scripts parse CLI output.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: output parses with jq.

### Requirements

- [ ] Add `--json`
- [ ] Keep logs on stderr
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Output parses with jq
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add a config file loader to the CLI

**Labels:** sdk, tooling, phase-2, good first issue

### Summary

Repeating flags is tedious.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: precedence order is tested.

### Requirements

- [ ] Support `sforge.config.json`
- [ ] Merge with env and flags
- [ ] Document precedence

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Precedence order is tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add shell completions for the CLI

**Labels:** sdk, tooling, phase-2, good first issue

### Summary

Discoverability.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: completion file builds in CI.

### Requirements

- [ ] Generate bash, zsh and fish completions
- [ ] Document install
- [ ] Test generation

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Completion file builds in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add unit tests for sponsored.ts fee bump paths

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

Sponsored writes are security sensitive.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: coverage report shows target.

### Requirements

- [ ] Test fee source, sequence handling and failure paths
- [ ] Mock the RPC
- [ ] Reach 90% coverage on the file

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Coverage report shows target
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add coverage reporting and a threshold to SDK CI

**Labels:** sdk, ci, testing, phase-2, good first issue

### Summary

No coverage is tracked.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: pR fails below threshold.

### Requirements

- [ ] Enable vitest coverage
- [ ] Upload to Codecov
- [ ] Set an 85% line threshold

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] PR fails below threshold
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add mutation testing to utils.ts

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

Amount parsing bugs (IR-06) show the value of strong tests.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: mutation score above 80%.

### Requirements

- [ ] Run Stryker on `utils.ts`
- [ ] Kill surviving mutants
- [ ] Report score

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Mutation score above 80%
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add contract parity tests between SDK math and Rust math

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

The oracle parity table shows the pattern works.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: fixtures regenerate with no diff.

### Requirements

- [ ] Extend the parity fixture approach to other shared math
- [ ] Regenerate in CI
- [ ] Fail on drift

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Fixtures regenerate with no diff
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add semantic-release or changesets for SDK versioning

**Labels:** sdk, ci, phase-2, difficulty: medium

### Summary

Versions are bumped manually.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: merging a changeset opens a release PR.

### Requirements

- [ ] Add changesets
- [ ] Add a release PR workflow
- [ ] Document usage

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Merging a changeset opens a release PR
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`

---

## Add a deprecation policy and helpers to the SDK

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Breaking changes (IR-06) need a migration path.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: policy merged.

### Requirements

- [ ] Document a two-minor-version deprecation window
- [ ] Add `@deprecated` tags with replacements
- [ ] Add a lint rule

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Policy merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add an upgrade guide template for breaking SDK releases

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Consumers need a standard format.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: guide exists.

### Requirements

- [ ] Create `sdk/docs/migrations/`
- [ ] Write 0.2 to 0.3 guide
- [ ] Link from the changelog

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Guide exists
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add telemetry hooks (opt-in) for client calls

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Integrators want latency metrics.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: hook receives method and duration.

### Requirements

- [ ] Add an `onCall` hook with timing data
- [ ] Keep it off by default
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Hook receives method and duration
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add structured logging with a pluggable logger

**Labels:** sdk, phase-2, good first issue

### Summary

The SDK should not use console directly.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: custom logger receives debug lines.

### Requirements

- [ ] Define a Logger interface
- [ ] Default to silent
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Custom logger receives debug lines
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a mock client for application unit tests

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

Apps need to test without a network.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: example app test uses the mock.

### Requirements

- [ ] Ship `@stellarforge-protocol/sdk/testing` with in-memory fakes
- [ ] Cover rwa-asset and compliance
- [ ] Document

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Example app test uses the mock
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add React hooks package for the SDK

**Labels:** sdk, frontend, phase-2, difficulty: hard, help wanted

### Summary

React dApps want hooks.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: hooks render in a sample app.

### Requirements

- [ ] Create `packages/react` with `useBalance`, `useKycStatus`, `useProposals`
- [ ] Use TanStack Query
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Hooks render in a sample app
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [ROADMAP.md](ROADMAP.md)

---

## Add a Node.js indexer reference using the SDK

**Labels:** sdk, tooling, phase-2, difficulty: hard, help wanted

### Summary

There is no reference for event indexing.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: indexer syncs testnet events.

### Requirements

- [ ] Build a small indexer writing to SQLite
- [ ] Replay from a cursor
- [ ] Document schema

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Indexer syncs testnet events
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add a Postgres schema for protocol events

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Production indexers need a standard schema.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: schema applies on Postgres 16.

### Requirements

- [ ] Define tables for transfers, mints, KYC and votes
- [ ] Add migrations
- [ ] Document

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Schema applies on Postgres 16
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a GraphQL API over the indexer

**Labels:** tooling, phase-2, difficulty: hard, help wanted

### Summary

Frontends prefer GraphQL.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: query returns holders for an asset.

### Requirements

- [ ] Expose balances, holders and proposals
- [ ] Add pagination
- [ ] Add tests

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Query returns holders for an asset
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add smoke test coverage for the oracle client

**Labels:** sdk, testing, phase-2, good first issue

### Summary

The testnet smoke tests may not cover the oracle.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: nightly workflow includes oracle.

### Requirements

- [ ] Add read and write steps
- [ ] Use a throwaway oracle
- [ ] Run in the nightly workflow

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Nightly workflow includes oracle
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Fix SDK README accuracy against the current API

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

The README may mention outdated methods after 0.3.0.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: snippets compile.

### Requirements

- [ ] Audit every snippet
- [ ] Compile snippets in CI
- [ ] Update outdated ones

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Snippets compile
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a recipe docs section: issue a tokenized bond

**Labels:** documentation, sdk, phase-2, good first issue

### Summary

Developers learn from scenarios.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: guide code runs on testnet.

### Requirements

- [ ] Write an end to end guide with SDK code
- [ ] Include compliance setup
- [ ] Test the code

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Guide code runs on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a recipe docs section: tokenized real estate with KYC

**Labels:** documentation, sdk, phase-2, good first issue

### Summary

A second flagship scenario.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: guide code runs on testnet.

### Requirements

- [ ] Write the guide
- [ ] Include registry listing
- [ ] Test the code

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Guide code runs on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a recipe docs section: DAO vote on an asset parameter

**Labels:** documentation, sdk, phase-2, good first issue

### Summary

Governance usage is undocumented.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: guide code runs on testnet.

### Requirements

- [ ] Write the guide
- [ ] Use propose, vote and finalize
- [ ] Test the code

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Guide code runs on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add SDK security guide: key handling and signing

**Labels:** documentation, security, phase-2, good first issue

### Summary

Wrong key handling is the top integrator risk.

### Context

`@stellarforge-protocol/sdk` (0.3.0) ships clients for rwa-asset, compliance, registry, governance and oracle-adapter, with sponsored writes. This stage raises it to a stable 1.0 developer experience.

This issue belongs to **Stage 3: TypeScript SDK Maturity**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Document environment secrets, signer adapters and fee sponsors
- [ ] Add a checklist
- [ ] Link from README

### Out of Scope

Contract changes. SDK work here must target the deployed ABI.

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

<!-- Stage 4: Phase 2 - Fractionalization Vault -->

## Write ADR-006: vault share accounting model

**Labels:** documentation, contracts, phase-2, difficulty: medium

### Summary

Share math decides rounding, inflation and donation risks. Decide it before code.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: aDR merged with a decision.

### Requirements

- [ ] Compare ERC-4626 style shares with fixed ratio shares
- [ ] Decide rounding direction
- [ ] Record first-depositor inflation mitigation

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] ADR merged with a decision
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the vault crate in the workspace

**Labels:** contracts, phase-2, good first issue

### Summary

No `contracts/vault` crate exists.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: `cargo test -p vault` runs.

### Requirements

- [ ] Create `Cargo.toml`, `src/lib.rs`, `error.rs`, `events.rs`
- [ ] Add to workspace and Makefile build
- [ ] Add a constructor test

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] `cargo test -p vault` runs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Define vault storage keys and TTL policy

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Storage tiers must follow ADR-001.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: every key has a tier and TTL rule.

### Requirements

- [ ] List instance, persistent and temporary keys
- [ ] Reuse `contracts/common` TTL helpers
- [ ] Document in the ADR

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Every key has a tier and TTL rule
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the vault constructor with underlying asset config

**Labels:** contracts, phase-2, difficulty: medium

### Summary

The vault needs admin, underlying token, share metadata and cap.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: constructor test with auth passes.

### Requirements

- [ ] Accept admin, underlying, name, symbol, decimals, share cap
- [ ] Require admin auth (see constructor auth tests)
- [ ] Reject zero cap

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Constructor test with auth passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement share token interface (SEP-41) inside the vault

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Shares must be transferable and visible in wallets.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: sEP-41 client works against the vault.

### Requirements

- [ ] Implement balance, transfer, approve, transfer_from, burn
- [ ] Reuse patterns from rwa-asset
- [ ] Emit SEP-41 events

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] SEP-41 client works against the vault
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement deposit with pro-rata share minting

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Core feature of the vault.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: deposit then redeem returns the same amount within rounding.

### Requirements

- [ ] Pull underlying from the depositor
- [ ] Mint shares by the chosen formula
- [ ] Emit `Deposit`

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Deposit then redeem returns the same amount within rounding
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement redeem with pro-rata underlying return

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Inverse of deposit.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: share supply and holdings reconcile.

### Requirements

- [ ] Burn shares
- [ ] Transfer underlying
- [ ] Emit `Redeem`

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Share supply and holdings reconcile
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement preview_deposit and preview_redeem

**Labels:** contracts, phase-2, good first issue

### Summary

Frontends need quotes without state change.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: preview equals execution in tests.

### Requirements

- [ ] Add pure view functions
- [ ] Match actual execution exactly
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Preview equals execution in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement max_deposit and max_redeem views

**Labels:** contracts, phase-2, good first issue

### Summary

Limits (cap, lock-ups) must be visible.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: values match accepted amounts.

### Requirements

- [ ] Return per-user maxima
- [ ] Account for cap and lock-ups
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Values match accepted amounts
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Enforce the maximum fractional share supply

**Labels:** contracts, phase-2, good first issue

### Summary

PRD requires a cap.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: deposit at cap passes, cap plus 1 fails.

### Requirements

- [ ] Reject deposits over the cap
- [ ] Return a named error
- [ ] Add a boundary test

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Deposit at cap passes, cap plus 1 fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the lock-up period per vault

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Redemption must respect lock-ups.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: redeem one ledger early fails.

### Requirements

- [ ] Record deposit ledger per user
- [ ] Reject early redeem
- [ ] Add tests around the boundary

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Redeem one ledger early fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Decide lock-up semantics for transferred shares

**Labels:** contracts, documentation, phase-2, difficulty: medium

### Summary

A user could dodge lock-up by transferring shares.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: documented behavior is tested.

### Requirements

- [ ] Choose per-account or per-lot tracking
- [ ] Record in the ADR
- [ ] Implement the choice

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Documented behavior is tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Integrate the oracle-adapter for NAV

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Share value depends on NAV.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: nAV used by `nav_per_share`.

### Requirements

- [ ] Call `lastprice` of the oracle
- [ ] Reject stale prices
- [ ] Use fixed-point math

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] NAV used by `nav_per_share`
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a staleness check for oracle prices in the vault

**Labels:** contracts, security, phase-2, difficulty: medium

### Summary

Old prices cause mispricing.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: stale price reverts.

### Requirements

- [ ] Add `max_oracle_age`
- [ ] Reject deposits and redeems when stale
- [ ] Add tests with advancing ledgers

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Stale price reverts
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement nav_per_share view

**Labels:** contracts, phase-2, good first issue

### Summary

Useful to UIs and indexers.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: result matches a hand-computed table.

### Requirements

- [ ] Compute from NAV and supply
- [ ] Handle zero supply
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Result matches a hand-computed table
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add vault pause and emergency withdraw policy

**Labels:** contracts, security, phase-2, difficulty: medium

### Summary

Failure handling for the vault.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: policy tested.

### Requirements

- [ ] Add admin pause
- [ ] Define whether redeem remains open during pause
- [ ] Document

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Policy tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Gate vault deposits through the compliance contract

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Fractional holders require KYC.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: non-KYC deposit fails.

### Requirements

- [ ] Call `is_compliant` on deposit and share transfer
- [ ] Make the compliance contract optional
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Non-KYC deposit fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a deposit fee and redemption fee option

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Issuers may charge fees.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: fee accounting reconciles in tests.

### Requirements

- [ ] Add fee bps with a maximum of 500
- [ ] Send fees to a recipient
- [ ] Emit fee data

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Fee accounting reconciles in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a management fee accrual option

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Annual fees on NAV are common.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: accrued fee matches a reference calculation.

### Requirements

- [ ] Accrue per ledger
- [ ] Mint fee shares to a recipient
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Accrued fee matches a reference calculation
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add admin role transfer with dual authorization to the vault

**Labels:** contracts, security, phase-2, good first issue

### Summary

All contracts follow IR-04.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: single-signature rotation is rejected.

### Requirements

- [ ] Add `transfer_admin` and `AdminTransferred`
- [ ] Add tests per IR-04 pattern
- [ ] Document

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Single-signature rotation is rejected
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add vault constructor auth test

**Labels:** contracts, testing, phase-2, good first issue

### Summary

IR-17 requires one per crate.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: test present and mutation-checked.

### Requirements

- [ ] Create `tests/test_constructor_auth.rs`
- [ ] Assert deployment fails without admin auth
- [ ] Verify it fails when auth is removed

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Test present and mutation-checked
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Write vault unit tests for deposit and redeem rounding

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Rounding bugs steal value.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: no rounding sequence reduces other holders' value.

### Requirements

- [ ] Test with tiny and huge amounts
- [ ] Test first deposit
- [ ] Assert vault never loses value on rounding

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] No rounding sequence reduces other holders' value
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Write property tests for the vault share invariant

**Labels:** contracts, testing, phase-2, difficulty: hard, help wanted

### Summary

Invariants find rare bugs.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: invariant holds.

### Requirements

- [ ] Randomize deposit, redeem and transfer
- [ ] Assert assets per share never decreases except by NAV change
- [ ] Run 10,000 cases

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Invariant holds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test the first-depositor inflation attack

**Labels:** contracts, security, testing, phase-2, difficulty: medium

### Summary

A known ERC-4626 class attack.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: attacker cannot steal victim deposit.

### Requirements

- [ ] Reproduce the attack in a test
- [ ] Implement the chosen mitigation
- [ ] Prove the attack fails

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Attacker cannot steal victim deposit
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test vault behavior with a donation of underlying tokens

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Direct transfers change the ratio.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: donation does not break share price math.

### Requirements

- [ ] Add a donation test
- [ ] Decide if the vault tracks internal accounting
- [ ] Document

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Donation does not break share price math
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test vault with a rwa-asset that has compliance enabled

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Integration between two real contracts.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: test passes with compliance on.

### Requirements

- [ ] Deploy both in a test env
- [ ] Run deposit and redeem
- [ ] Assert compliance calls

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Test passes with compliance on
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test vault with a paused underlying asset

**Labels:** contracts, testing, phase-2, good first issue

### Summary

The underlying may be paused.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: behavior documented and tested.

### Requirements

- [ ] Pause the asset
- [ ] Attempt deposit and redeem
- [ ] Assert clean errors

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Behavior documented and tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test vault under mainnet resource limits

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Cross-contract calls cost more.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: counts recorded in the baseline.

### Requirements

- [ ] Run deposit and redeem with mainnet budgets
- [ ] Record instruction counts
- [ ] Add a budget assertion

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Counts recorded in the baseline
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add vault events and pin their wire format

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Matches IR-03 practice.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: snapshot tests exist.

### Requirements

- [ ] Define events for deposit, redeem, fee, pause
- [ ] Snapshot test each
- [ ] Document in crate README

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Snapshot tests exist
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add vault to the deploy script

**Labels:** tooling, phase-2, good first issue

### Summary

`scripts/deploy.sh` handles four or five contracts.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: script deploys the vault on testnet.

### Requirements

- [ ] Add the vault with its env variables
- [ ] Validate underlying and oracle addresses
- [ ] Update `.env.example`

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Script deploys the vault on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add vault to the registry as a distinct asset class

**Labels:** contracts, phase-2, good first issue

### Summary

Frontends need to tell vaults from assets.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: registry stores a vault entry.

### Requirements

- [ ] Add a class value for fractional vaults
- [ ] Update registry tests
- [ ] Update docs

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Registry stores a vault entry
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Complete the VaultClient in the SDK

**Labels:** sdk, phase-2, difficulty: medium

### Summary

`sdk/src/vault.ts` exists but needs full coverage.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: all contract functions are wrapped.

### Requirements

- [ ] Wrap deposit, redeem, previews, views and admin methods
- [ ] Add typed errors
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] All contract functions are wrapped
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add SDK write-path helpers for vault deposit and redeem

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Matches the Phase 1 write helpers.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: deposit works on testnet in the write smoke test.

### Requirements

- [ ] Add builders with simulation and submit
- [ ] Support sponsored mode
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Deposit works on testnet in the write smoke test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add vault event decoders to the SDK

**Labels:** sdk, phase-2, good first issue

### Summary

Consumers decode vault events.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: decoders handle recorded events.

### Requirements

- [ ] Add typed decoders
- [ ] Add fixtures
- [ ] Export types

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Decoders handle recorded events
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add vault end to end testnet smoke test

**Labels:** sdk, testing, phase-2, difficulty: medium

### Summary

A real network proves the integration.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: smoke test is green.

### Requirements

- [ ] Deploy asset, oracle and vault
- [ ] Deposit and redeem
- [ ] Run in the nightly workflow

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Smoke test is green
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Write the vault integration guide

**Labels:** documentation, phase-2, good first issue

### Summary

Integrators need a guide.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Cover deploy, oracle setup, compliance and lock-ups
- [ ] Include code
- [ ] Link from README

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write the vault security review checklist

**Labels:** documentation, security, phase-2, difficulty: medium

### Summary

Prepare for the external audit.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: checklist merged.

### Requirements

- [ ] List invariants, assumptions and known limits
- [ ] Link tests
- [ ] Add to the audit handover

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Checklist merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add a real estate demo using the vault

**Labels:** documentation, tooling, phase-2, difficulty: medium

### Summary

PRD exit criteria call for a real estate test on testnet.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: demo script runs from a clean testnet.

### Requirements

- [ ] Script a demo: mint a property token, deposit, issue shares
- [ ] Document steps
- [ ] Capture outputs

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Demo script runs from a clean testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `scripts/`
- [Makefile](Makefile)

---

## Evaluate vault share transfer restrictions for accredited investors

**Labels:** contracts, documentation, phase-2, difficulty: medium

### Summary

Some issuers limit share holders.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: decision recorded.

### Requirements

- [ ] Describe options using compliance levels
- [ ] Choose and implement or defer
- [ ] Record in the ADR

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Decision recorded
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a view that reports total assets under management

**Labels:** contracts, phase-2, good first issue

### Summary

Dashboards need AUM.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: value matches the oracle price times holdings.

### Requirements

- [ ] Add `total_assets`
- [ ] Use NAV times underlying balance
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] Value matches the oracle price times holdings
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add vault wind-down and final settlement mode

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Vaults must be closable.

### Context

Phase 2 adds a `vault` contract that locks an underlying RWA asset and issues fractional shares, priced by the oracle-adapter, with lock-ups. `sdk/src/vault.ts` already exists as a client stub. See ROADMAP.md Phase 2 and PRD sections 5.4 and FR-06.

This issue belongs to **Stage 4: Phase 2 - Fractionalization Vault**.

### Objective

Deliver the change described above so that: all holders redeem after settlement.

### Requirements

- [ ] Add a settle function that opens redemption at a final NAV
- [ ] Block deposits
- [ ] Add tests

### Out of Scope

Yield distribution (own stage) and tranching (Phase 3).

### Acceptance Criteria

- [ ] All holders redeem after settlement
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

<!-- Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness -->

## Write ADR-007: yield distribution snapshot model

**Labels:** documentation, contracts, phase-2, difficulty: medium

### Summary

Snapshot-at-ledger distribution can be done by checkpoints, merkle roots or pull-based accumulators.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Compare the three on cost and complexity
- [ ] Decide
- [ ] Record limits on holder count

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the yield-distributor crate

**Labels:** contracts, phase-2, good first issue

### Summary

No crate exists.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: `cargo test -p yield-distributor` runs.

### Requirements

- [ ] Create crate files, add to workspace and Makefile
- [ ] Add a constructor test

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] `cargo test -p yield-distributor` runs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement balance checkpoints in rwa-asset or vault shares

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Snapshots need historical balances.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: historical balance query is correct in tests.

### Requirements

- [ ] Add checkpoint writes on balance change
- [ ] Add `balance_at(ledger)`
- [ ] Bound storage growth

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Historical balance query is correct in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement announce_distribution with snapshot ledger

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Admin announces income to share.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: announcement stores the snapshot.

### Requirements

- [ ] Record asset, amount, snapshot ledger and total supply at snapshot
- [ ] Pull funds from the announcer
- [ ] Emit `DistributionAnnounced`

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Announcement stores the snapshot
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement claim for a single distribution

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Holders pull their share.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: claim twice reverts.

### Requirements

- [ ] Compute pro-rata share from the snapshot
- [ ] Mark as claimed
- [ ] Transfer funds

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Claim twice reverts
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement claim_many across distributions

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Users claim all outstanding in one transaction.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: claiming 10 distributions succeeds in one call.

### Requirements

- [ ] Accept a bounded list of ids
- [ ] Skip claimed ones safely
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Claiming 10 distributions succeeds in one call
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement multi-currency payouts

**Labels:** contracts, phase-2, difficulty: medium

### Summary

PRD requires XLM, USDC and custom assets.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: a distribution pays in USDC and another in XLM.

### Requirements

- [ ] Store the payout token per distribution
- [ ] Support any SEP-41 token
- [ ] Add tests with two tokens

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] A distribution pays in USDC and another in XLM
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Handle XLM through the native asset contract wrapper

**Labels:** contracts, phase-2, good first issue

### Summary

XLM is a Stellar Asset Contract on Soroban.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: xLM distribution works in tests.

### Requirements

- [ ] Test with the native SAC
- [ ] Document the address lookup
- [ ] Add an example

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] XLM distribution works in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the unclaimed yield reclaim timeout

**Labels:** contracts, phase-2, difficulty: medium

### Summary

PRD requires unclaimed funds to revert to the pool.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: reclaim before timeout fails.

### Requirements

- [ ] Add `reclaim_after_ledgers`
- [ ] Admin reclaims after the timeout
- [ ] Emit event

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Reclaim before timeout fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Round dust handling for pro-rata payouts

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Integer division leaves dust.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: total paid never exceeds announced amount.

### Requirements

- [ ] Define who receives dust
- [ ] Track it
- [ ] Test totals

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Total paid never exceeds announced amount
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Test distribution to 1,000 simulated holders

**Labels:** contracts, testing, phase-2, difficulty: hard, help wanted

### Summary

Phase 2 exit criteria demand it.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: sum paid equals sum due.

### Requirements

- [ ] Generate 1,000 holders in a test
- [ ] Run claims
- [ ] Assert the sum

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Sum paid equals sum due
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test distribution to 10,000 holders under resource limits

**Labels:** contracts, testing, phase-2, difficulty: hard, help wanted

### Summary

Scale check.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: numbers recorded.

### Requirements

- [ ] Run an ignored test
- [ ] Record instruction counts per claim
- [ ] Document the practical limit

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Numbers recorded
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Gate claims through compliance

**Labels:** contracts, phase-2, difficulty: medium

### Summary

A holder who lost KYC may be barred from payouts.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: expired KYC holder claim behavior is tested.

### Requirements

- [ ] Optionally call compliance on claim
- [ ] Document held-over funds
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Expired KYC holder claim behavior is tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add distribution history views

**Labels:** contracts, phase-2, good first issue

### Summary

PRD requires transparent history.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: history is queryable.

### Requirements

- [ ] Add `distribution_count` and `get_distribution`
- [ ] Paginate
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] History is queryable
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add yield-distributor events and snapshot tests

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Matches IR-03 practice.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: tests exist.

### Requirements

- [ ] Define events
- [ ] Snapshot each
- [ ] Document

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Tests exist
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add admin rotation and constructor auth tests to yield-distributor

**Labels:** contracts, testing, security, phase-2, good first issue

### Summary

Matches IR-04 and IR-17.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: tests mutation-checked.

### Requirements

- [ ] Add `transfer_admin`
- [ ] Add constructor auth test
- [ ] Document

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Tests mutation-checked
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Fuzz the distribution math

**Labels:** contracts, testing, security, phase-2, difficulty: hard, help wanted

### Summary

Pro-rata math is error prone.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: no violation in 100,000 cases.

### Requirements

- [ ] Property test: payouts never exceed funds
- [ ] Include zero balances and huge supply
- [ ] Run nightly

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] No violation in 100,000 cases
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add YieldDistributorClient to the SDK

**Labels:** sdk, phase-2, difficulty: medium

### Summary

ROADMAP lists vault and yield clients.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: client covers the contract ABI.

### Requirements

- [ ] Wrap all functions
- [ ] Add typed errors and events
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Client covers the contract ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add write helpers for announce and claim

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Matches write-path helpers.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: testnet write smoke test passes.

### Requirements

- [ ] Add builders with simulation, submission and sponsored mode
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Testnet write smoke test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add SDK helper to compute claimable amounts off-chain

**Labels:** sdk, phase-2, difficulty: medium

### Summary

UIs show claimables without a transaction.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: matches contract in parity tests.

### Requirements

- [ ] Read checkpoint balance and supply
- [ ] Replicate contract math
- [ ] Add parity fixtures

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Matches contract in parity tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add yield-distributor to the deploy script and manifest

**Labels:** tooling, phase-2, good first issue

### Summary

Deployment must include it.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: script deploys it.

### Requirements

- [ ] Add env variables
- [ ] Update `.env.example`
- [ ] Update the manifest writer

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Script deploys it
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Write the yield distribution operator guide

**Labels:** documentation, phase-2, good first issue

### Summary

Issuers need steps for periodic income.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Describe announcing, funding, claim windows and reclaim
- [ ] Include CLI examples

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write the yield-distributor security checklist

**Labels:** documentation, security, phase-2, difficulty: medium

### Summary

Prepare for audit.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: checklist merged.

### Requirements

- [ ] List invariants and assumptions
- [ ] Link tests
- [ ] Add to handover

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Checklist merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Complete the oracle-adapter SDK client coverage

**Labels:** sdk, phase-2, difficulty: medium

### Summary

ROADMAP lists SDK and deploy script for the oracle as open.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: client covers all 17 functions.

### Requirements

- [ ] Wrap every function including `report` and `override_price`
- [ ] Add typed errors
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Client covers all 17 functions
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add oracle-adapter to scripts/deploy.sh

**Labels:** tooling, phase-2, good first issue

### Summary

ROADMAP lists it as open.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: script deploys the oracle.

### Requirements

- [ ] Add env variables for base asset, decimals and resolution
- [ ] Validate inputs
- [ ] Update `.env.example`

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Script deploys the oracle
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Write a reporter operations guide for the oracle-adapter

**Labels:** documentation, phase-2, good first issue

### Summary

Reporters need key and cadence advice.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Describe cadence, deviation limits and rotation
- [ ] Link ADR-005

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a Reflector oracle wrapper per ADR-005

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

ADR-005 plans wrappers for external feeds.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: wrapper returns the mock price.

### Requirements

- [ ] Implement a read-through wrapper to Reflector
- [ ] Match SEP-40
- [ ] Add tests with a mock

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Wrapper returns the mock price
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a Pyth wrapper per ADR-005

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

Same, for Pyth.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: wrapper tested with mock.

### Requirements

- [ ] Implement the wrapper
- [ ] Handle confidence intervals
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Wrapper tested with mock
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a median aggregator over several oracles

**Labels:** contracts, phase-2, difficulty: hard, help wanted

### Summary

One feed is a single point of failure.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: outlier is ignored.

### Requirements

- [ ] Accept up to 5 feeds
- [ ] Return the median
- [ ] Ignore stale feeds
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Outlier is ignored
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Automate testnet deployment with deterministic contract IDs

**Labels:** tooling, phase-2, difficulty: hard, help wanted

### Summary

ROADMAP lists it as open.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: two runs yield the same IDs.

### Requirements

- [ ] Use salted deployer addresses
- [ ] Record IDs in a manifest
- [ ] Add an idempotent rerun mode

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Two runs yield the same IDs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a post-deploy verification script

**Labels:** tooling, security, phase-2, difficulty: medium

### Summary

Deployments must be checked.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: script flags a wrong hash.

### Requirements

- [ ] Compare on-chain WASM hash with the release artifact
- [ ] Check admin and config
- [ ] Fail loudly

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Script flags a wrong hash
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add a deployment dry-run mode

**Labels:** tooling, phase-2, good first issue

### Summary

Operators want a plan before spending.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: dry-run submits nothing.

### Requirements

- [ ] Print every command and fee estimate
- [ ] Do not submit
- [ ] Add tests

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Dry-run submits nothing
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a contract upgrade runbook with test on testnet

**Labels:** documentation, contracts, phase-2, difficulty: medium

### Summary

ADR-003 defines the path.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: runbook merged with a real run.

### Requirements

- [ ] Execute an upgrade on testnet
- [ ] Document each step
- [ ] Capture the verification

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Runbook merged with a real run
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a storage migration test harness

**Labels:** contracts, testing, phase-2, difficulty: hard, help wanted

### Summary

Upgrades may change storage layouts.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: harness used in CI.

### Requirements

- [ ] Build a harness that loads old state and runs new code
- [ ] Add one sample migration
- [ ] Document

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Harness used in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Prepare the Phase 1 mainnet launch checklist

**Labels:** documentation, security, phase-2, difficulty: medium

### Summary

ROADMAP plans mainnet after audit.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: checklist reviewed.

### Requirements

- [ ] List audit, keys, monitoring and communications
- [ ] Assign owners

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Checklist reviewed
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Define a mainnet key ceremony procedure

**Labels:** documentation, security, phase-2, difficulty: medium

### Summary

Admin keys need a safe process.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: rehearsal done.

### Requirements

- [ ] Describe multisig setup and hardware signers
- [ ] Record backups
- [ ] Add a rehearsal on testnet

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Rehearsal done
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Set up monitoring and alerting for deployed contracts

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Production needs alerts.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: test alert fires on testnet.

### Requirements

- [ ] Alert on admin actions, pauses and large mints
- [ ] Use the indexer events
- [ ] Document

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Test alert fires on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Create a public status page for protocol contracts

**Labels:** tooling, frontend, phase-2, difficulty: medium

### Summary

Transparency.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: page builds in CI.

### Requirements

- [ ] List contract IDs, versions and WASM hashes
- [ ] Generate from the manifest
- [ ] Host as static page

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Page builds in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [ROADMAP.md](ROADMAP.md)

---

## Draft the Phase 2 external audit scope

**Labels:** documentation, security, phase-2, difficulty: medium

### Summary

ROADMAP lists audit for Phase 1 and 2.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] List contracts, commits and questions
- [ ] Pick candidate firms
- [ ] Estimate effort

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Run a testnet incentive and bug-hunt event

**Labels:** documentation, phase-2, good first issue

### Summary

Real usage finds bugs.

### Context

Phase 2 also delivers the `yield-distributor` contract, finishes the oracle-adapter rollout (SDK and deploy script items remain open in ROADMAP.md), automates testnet deployment and prepares the Phase 1 mainnet launch.

This issue belongs to **Stage 5: Phase 2 - Yield Distributor, Oracle Follow-ups and Mainnet Readiness**.

### Objective

Deliver the change described above so that: report published.

### Requirements

- [ ] Write rules and rewards
- [ ] Prepare faucets and scripts
- [ ] Publish a report

### Out of Scope

Tranching, governance execution hooks and cross-chain work.

### Acceptance Criteria

- [ ] Report published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution -->

## Write ADR-008: tranche waterfall model

**Labels:** documentation, contracts, phase-3, difficulty: medium

### Summary

Waterfall order and loss allocation define the product.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Specify senior, mezzanine and junior payment order
- [ ] Define loss absorption
- [ ] Define rounding rules

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the tranche crate

**Labels:** contracts, phase-3, good first issue

### Summary

No crate exists.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: `cargo test -p tranche` runs.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace and Makefile
- [ ] Add a constructor test

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] `cargo test -p tranche` runs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Define the tranche pool configuration struct

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Pools need class parameters.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: invalid configs are rejected.

### Requirements

- [ ] Add class count, target yield bps, size caps
- [ ] Validate sum rules
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Invalid configs are rejected
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement tranche token issuance per class

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Each class is a transferable token.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: class tokens appear in a wallet.

### Requirements

- [ ] Mint class tokens on deposit
- [ ] Reuse SEP-41 patterns
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Class tokens appear in a wallet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement deposit into a chosen tranche class

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Users pick risk.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: deposit mints the right class tokens.

### Requirements

- [ ] Accept class id and amount
- [ ] Enforce class caps
- [ ] Emit events

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Deposit mints the right class tokens
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the yield waterfall distribution

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Senior is paid first.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: senior receives target before junior receives anything.

### Requirements

- [ ] Apply yield by class priority up to target
- [ ] Pass excess to junior
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Senior receives target before junior receives anything
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement loss absorption from the junior tranche up

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

First-loss design.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: loss of 10% hits junior only when junior holds 20%.

### Requirements

- [ ] Apply losses junior first
- [ ] Track impairment per class
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Loss of 10% hits junior only when junior holds 20%
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement default reporting by an authorized role

**Labels:** contracts, security, phase-3, difficulty: medium

### Summary

Losses need a trusted input.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: unauthorized report fails.

### Requirements

- [ ] Add a loss reporter role
- [ ] Emit events
- [ ] Bound a single report

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Unauthorized report fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement tranche redemption with a queue

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Withdrawals may exceed liquidity.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: queue pays in order.

### Requirements

- [ ] Add a FIFO queue
- [ ] Fulfill as liquidity arrives
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Queue pays in order
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add tranche NAV per class using the oracle

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Share value per class.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: nAV is consistent with the waterfall.

### Requirements

- [ ] Compute class NAV after waterfall and losses
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] NAV is consistent with the waterfall
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add tranche pause and wind-down

**Labels:** contracts, security, phase-3, difficulty: medium

### Summary

Failure handling.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: wind-down distributes by seniority.

### Requirements

- [ ] Add pause
- [ ] Add wind-down mode with pro-rata by seniority
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Wind-down distributes by seniority
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Gate tranche deposits through compliance

**Labels:** contracts, phase-3, good first issue

### Summary

Regulated products.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: non-compliant deposit fails.

### Requirements

- [ ] Optionally require a compliance level per class
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Non-compliant deposit fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add tranche events and snapshot tests

**Labels:** contracts, testing, phase-3, good first issue

### Summary

Matches IR-03 practice.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: tests exist.

### Requirements

- [ ] Define events
- [ ] Snapshot them
- [ ] Document

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Tests exist
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add admin rotation and constructor auth tests to tranche

**Labels:** contracts, testing, security, phase-3, good first issue

### Summary

Matches IR-04 and IR-17.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: tests mutation-checked.

### Requirements

- [ ] Add `transfer_admin`
- [ ] Add constructor auth test

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Tests mutation-checked
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Property test the waterfall conservation

**Labels:** contracts, testing, phase-3, difficulty: hard, help wanted

### Summary

Money must not appear or vanish.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: no violation in 100,000 cases.

### Requirements

- [ ] Randomize yields, losses and deposits
- [ ] Assert total in equals total out
- [ ] Run nightly

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] No violation in 100,000 cases
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Simulate defaults against the tranche contract

**Labels:** contracts, testing, phase-3, difficulty: medium

### Summary

Phase 3 exit criteria demand it.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: contract matches the model.

### Requirements

- [ ] Script several default scenarios
- [ ] Assert outcomes against a spreadsheet model
- [ ] Store the model

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Contract matches the model
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add TrancheClient to the SDK

**Labels:** sdk, phase-3, difficulty: medium

### Summary

SDK parity.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Wrap all functions
- [ ] Add events and errors
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add tranche write helpers

**Labels:** sdk, phase-3, difficulty: medium

### Summary

SDK parity.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: write smoke test passes.

### Requirements

- [ ] Add builders and sponsored mode
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Write smoke test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add tranche to the deploy script and manifest

**Labels:** tooling, phase-3, good first issue

### Summary

Deployment.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: script deploys it.

### Requirements

- [ ] Add env variables
- [ ] Update `.env.example`

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Script deploys it
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Write the tranche issuer guide

**Labels:** documentation, phase-3, good first issue

### Summary

Issuers set up pools.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Cover parameter choice and reporting
- [ ] Include risk language

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write the tranche investor risk explainer

**Labels:** documentation, phase-3, good first issue

### Summary

Retail users need plain language.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: explainer merged.

### Requirements

- [ ] Explain senior, mezzanine and junior risk with examples
- [ ] Avoid advice
- [ ] Review with legal

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Explainer merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write the tranche security checklist

**Labels:** documentation, security, phase-3, difficulty: medium

### Summary

Audit preparation.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: checklist merged.

### Requirements

- [ ] List invariants and assumptions
- [ ] Link tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Checklist merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write ADR-009: DEX and AMM integration approach

**Labels:** documentation, contracts, phase-3, difficulty: medium

### Summary

The liquidity-pool adapter needs a design.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Compare wrapping Stellar liquidity pools and Soroban AMMs
- [ ] Decide on access control
- [ ] Document

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the liquidity-pool adapter crate

**Labels:** contracts, phase-3, good first issue

### Summary

No crate exists.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace
- [ ] Add a constructor test

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement deposit and withdraw through the adapter

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Core feature.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: deposit through the adapter works against a mock pool.

### Requirements

- [ ] Forward to the pool
- [ ] Enforce compliance on the beneficiary
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Deposit through the adapter works against a mock pool
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement swap through the adapter with allowlists

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Restrict who can trade.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: non-compliant swap fails.

### Requirements

- [ ] Check compliance for both sides
- [ ] Emit events
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Non-compliant swap fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add slippage and deadline protection to the adapter

**Labels:** contracts, security, phase-3, difficulty: medium

### Summary

Prevent sandwich losses.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: late swap reverts.

### Requirements

- [ ] Require min out and deadline
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Late swap reverts
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add adapter tests against Stellar classic liquidity pools

**Labels:** contracts, testing, phase-3, difficulty: hard, help wanted

### Summary

Real pool behavior.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: test run documented.

### Requirements

- [ ] Test on testnet with a real pool
- [ ] Record results

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Test run documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add a LiquidityPoolClient to the SDK

**Labels:** sdk, phase-3, difficulty: medium

### Summary

SDK parity.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Wrap functions
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Write ADR-010: governance execution hooks and timelock

**Labels:** documentation, contracts, security, phase-3, difficulty: hard, help wanted

### Summary

Passed proposals must call contracts safely.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Define the action format, timelock and guardian
- [ ] Analyze reentrancy and auth

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Extend proposals with executable actions

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Today proposals are text only.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: proposal stores an action.

### Requirements

- [ ] Add target contract, function and args to `Proposal`
- [ ] Bound sizes
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Proposal stores an action
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement queue and execute with a timelock

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Delay lets users react.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: early execute fails.

### Requirements

- [ ] Add `queue` after pass
- [ ] Add `execute` after the delay
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Early execute fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement a guardian cancel in the timelock window

**Labels:** contracts, security, phase-3, difficulty: medium

### Summary

Emergency stop.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: guardian cancel works inside the window.

### Requirements

- [ ] Add guardian role
- [ ] Add `cancel_queued`
- [ ] Emit event

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Guardian cancel works inside the window
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Restrict executable targets with an allowlist

**Labels:** contracts, security, phase-3, difficulty: medium

### Summary

Prevent arbitrary calls.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: disallowed target fails.

### Requirements

- [ ] Add an allowlist of targets and functions
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Disallowed target fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Make the governance contract the admin of other contracts

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Execution needs authority.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: proposal changes an asset parameter on testnet.

### Requirements

- [ ] Define the admin handoff flow using dual auth rotation
- [ ] Test with rwa-asset
- [ ] Document

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Proposal changes an asset parameter on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Prove a passed proposal executes a contract call on testnet

**Labels:** contracts, testing, phase-3, difficulty: medium

### Summary

Phase 3 exit criterion.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: evidence published.

### Requirements

- [ ] Script the full flow
- [ ] Record transaction hashes
- [ ] Add to docs

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Evidence published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance execution to the SDK

**Labels:** sdk, phase-3, difficulty: medium

### Summary

SDK parity.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: covers the new ABI.

### Requirements

- [ ] Add queue, execute and cancel methods
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Covers the new ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Write ADR-011: SFORGE token design

**Labels:** documentation, contracts, phase-3, difficulty: medium

### Summary

PRD says non-transferable voting power.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Specify earn rules, soulbound behavior and decay
- [ ] Compare to alternatives

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the sforge-token crate

**Labels:** contracts, phase-3, good first issue

### Summary

No crate exists.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate
- [ ] Add to workspace
- [ ] Add a constructor test

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement non-transferable balances and mint by role

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Soulbound voting power.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: transfer always fails.

### Requirements

- [ ] Disable transfer
- [ ] Add a minter role
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Transfer always fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement an earning rules module

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Power is earned through participation.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: caps hold.

### Requirements

- [ ] Record contributions via role-gated calls
- [ ] Cap per period
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Caps hold
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement vote weight snapshots for governance

**Labels:** contracts, phase-3, difficulty: hard, help wanted

### Summary

Prevents double voting through transfers and late mints.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: weight at proposal creation is used.

### Requirements

- [ ] Checkpoint balances
- [ ] Provide `weight_at(ledger)`
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Weight at proposal creation is used
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Connect governance `vote` to SFORGE weights

**Labels:** contracts, security, phase-3, difficulty: hard, help wanted

### Summary

Replace caller-supplied weight.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: forged weight is impossible.

### Requirements

- [ ] Read weight from the token at the snapshot
- [ ] Reject mismatches
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Forged weight is impossible
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add delegation to SFORGE voting power

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Delegates are common in DAOs.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: delegated weight counts once.

### Requirements

- [ ] Add delegate and undelegate
- [ ] Update checkpoints
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Delegated weight counts once
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Plan the SFORGE genesis distribution

**Labels:** documentation, phase-3, difficulty: medium

### Summary

Phase 3 exit criteria include distribution to contributors.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Define eligibility and snapshot rules
- [ ] Publish the list process
- [ ] Review

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add the protocol fee switch module

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Fee 0 to 50 bps, off by default.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: default is zero.

### Requirements

- [ ] Add a fee config controlled by governance
- [ ] Cap at 50 bps
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Default is zero
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Apply the protocol fee in vault and tranche

**Labels:** contracts, phase-3, difficulty: medium

### Summary

Fees apply across modules.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: fee flows are reconciled.

### Requirements

- [ ] Read the global fee
- [ ] Send to the treasury address
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Fee flows are reconciled
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a fee change event and a delay

**Labels:** contracts, security, phase-3, good first issue

### Summary

Users should see fee changes coming.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: immediate change fails.

### Requirements

- [ ] Require timelock for fee changes
- [ ] Emit events
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Immediate change fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Scaffold the Next.js reference dashboard

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Phase 3 lists a reference app.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: `npm run dev` shows a home page.

### Requirements

- [ ] Create `apps/dashboard` with Next.js and the SDK
- [ ] Add CI build
- [ ] Add README

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] `npm run dev` shows a home page
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add wallet connect to the dashboard

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Users need to sign.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: connect works on testnet.

### Requirements

- [ ] Integrate Freighter and one more wallet
- [ ] Show network and address

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Connect works on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add an asset list page from the registry

**Labels:** frontend, phase-3, good first issue

### Summary

Discovery.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: list renders testnet assets.

### Requirements

- [ ] Page through `list_assets`
- [ ] Show active flag
- [ ] Add loading states

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] List renders testnet assets
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add an asset detail page

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Asset view.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: detail page loads.

### Requirements

- [ ] Show metadata, supply, holders count and compliance level
- [ ] Add links to explorer

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Detail page loads
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add a holder portfolio page

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Users see holdings.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: page shows a test user's holdings.

### Requirements

- [ ] Show balances, claimable yield and lock-ups
- [ ] Add tests

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Page shows a test user's holdings
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add vault deposit and redeem forms

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Core UX.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: deposit completes on testnet.

### Requirements

- [ ] Use previews
- [ ] Show fees and lock-up
- [ ] Handle errors

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Deposit completes on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add a yield claim page

**Labels:** frontend, phase-3, good first issue

### Summary

Core UX.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: claim completes on testnet.

### Requirements

- [ ] List claimable distributions
- [ ] Add a claim all button

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Claim completes on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add governance proposal list and voting UI

**Labels:** frontend, phase-3, difficulty: medium

### Summary

Core UX.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: vote completes on testnet.

### Requirements

- [ ] List proposals
- [ ] Vote
- [ ] Show status

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Vote completes on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add a KYC status panel to the dashboard

**Labels:** frontend, phase-3, good first issue

### Summary

Users need to know their status.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: panel reads compliance data.

### Requirements

- [ ] Show level, jurisdiction and expiry
- [ ] Warn before expiry

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Panel reads compliance data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add accessibility checks to the dashboard

**Labels:** frontend, ci, phase-3, difficulty: medium

### Summary

WCAG matters for finance apps.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: no serious violations.

### Requirements

- [ ] Add axe tests
- [ ] Fix issues
- [ ] Run in CI

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] No serious violations
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- `.github/workflows/`

---

## Add dashboard i18n scaffolding

**Labels:** frontend, phase-3, good first issue

### Summary

Global audience.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: locale switch works.

### Requirements

- [ ] Add next-intl with English and one more locale
- [ ] Extract strings

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Locale switch works
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Add dashboard end to end tests with Playwright

**Labels:** frontend, testing, phase-3, difficulty: medium

### Summary

Prevent regressions.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: tests pass in CI.

### Requirements

- [ ] Test connect, deposit and vote against a mock chain
- [ ] Run in CI

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Tests pass in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Deploy the dashboard preview per pull request

**Labels:** frontend, ci, phase-3, difficulty: medium

### Summary

Reviewers need previews.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: preview URL appears on PRs.

### Requirements

- [ ] Add a preview workflow
- [ ] Comment the URL
- [ ] Use testnet only

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Preview URL appears on PRs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- `.github/workflows/`

---

## Write the Phase 3 external audit scope

**Labels:** documentation, security, phase-3, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] List contracts and commits
- [ ] Include timelock and token
- [ ] Estimate effort

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Prepare the Phase 2 mainnet deployment plan

**Labels:** documentation, phase-3, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 3 (target Q1 2027) adds the `tranche` contract, a liquidity-pool adapter, governance execution hooks with timelock, the `SFORGE` token, a governance-controlled protocol fee and a reference dashboard. See ROADMAP.md Phase 3 and PRD 5.6.

This issue belongs to **Stage 6: Phase 3 - DeFi Primitives, Tranching and Governance Execution**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Define order, keys and checks
- [ ] Reuse the Phase 1 checklist

### Out of Scope

Privacy, bridges and insurance (later phases).

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 7: Phase 4 - Privacy and Cross-Chain -->

## Write ADR-012: ZK proof system selection

**Labels:** documentation, contracts, research, phase-4, difficulty: hard, help wanted

### Summary

Groth16 and PLONK differ in setup, proof size and Soroban verification cost.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: aDR merged with benchmark data.

### Requirements

- [ ] Benchmark verification cost on Soroban
- [ ] Compare trusted setup needs
- [ ] Decide and record

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] ADR merged with benchmark data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype a Groth16 verifier in Soroban

**Labels:** contracts, research, phase-4, difficulty: hard, help wanted

### Summary

Verification cost decides feasibility.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: prototype verifies a known proof.

### Requirements

- [ ] Implement pairing checks using host crypto primitives
- [ ] Measure instructions
- [ ] Document limits

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Prototype verifies a known proof
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype a PLONK verifier in Soroban

**Labels:** contracts, research, phase-4, difficulty: hard, help wanted

### Summary

Compare with Groth16.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: prototype verifies a known proof.

### Requirements

- [ ] Implement the verifier
- [ ] Measure instructions
- [ ] Document limits

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Prototype verifies a known proof
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Scaffold the zk-verifier crate

**Labels:** contracts, phase-4, good first issue

### Summary

No crate exists.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace
- [ ] Add a constructor test

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Define the KYC circuit statement

**Labels:** research, documentation, phase-4, difficulty: hard, help wanted

### Summary

The proof must state what is proven.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: spec reviewed by a cryptographer.

### Requirements

- [ ] Specify public inputs: level at least N, not expired, issuer key
- [ ] Private inputs: identity attributes
- [ ] Write the spec

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Spec reviewed by a cryptographer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Implement the KYC circuit in Circom or Noir

**Labels:** research, phase-4, difficulty: hard, help wanted

### Summary

Circuit implementation.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: circuit tests pass.

### Requirements

- [ ] Implement the spec
- [ ] Add unit tests for the circuit
- [ ] Document constraints count

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Circuit tests pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Add a trusted setup or universal setup plan

**Labels:** research, security, phase-4, difficulty: hard, help wanted

### Summary

Groth16 needs a ceremony.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Design a multi-party ceremony or choose a universal setup
- [ ] Document verification

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Implement on-chain verifying key storage and rotation

**Labels:** contracts, security, phase-4, difficulty: medium

### Summary

Keys may need updating.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: rotation emits an event.

### Requirements

- [ ] Store keys by circuit id
- [ ] Allow admin or governance rotation
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Rotation emits an event
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement verify_proof entry point

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Core feature.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: valid proof passes, tampered proof fails.

### Requirements

- [ ] Accept proof and public inputs
- [ ] Return bool or error
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Valid proof passes, tampered proof fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Bind proofs to a ledger sequence for replay protection

**Labels:** contracts, security, phase-4, difficulty: medium

### Summary

PRD requires replay protection.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: expired proof is rejected.

### Requirements

- [ ] Include ledger window in public inputs
- [ ] Reject old proofs
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Expired proof is rejected
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add a nullifier set to prevent proof reuse

**Labels:** contracts, security, phase-4, difficulty: medium

### Summary

Prevent double use where required.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: reuse fails.

### Requirements

- [ ] Store nullifiers with TTL
- [ ] Reject duplicates
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Reuse fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Integrate ZK proofs into the compliance contract

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Proofs replace plain KYC records optionally.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: verified holder can transfer.

### Requirements

- [ ] Add `is_compliant_zk`
- [ ] Cache verified status with expiry
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Verified holder can transfer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Allow rwa-asset to accept ZK compliance

**Labels:** contracts, phase-4, difficulty: medium

### Summary

Asset contracts need the option.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: both modes work.

### Requirements

- [ ] Add a compliance mode flag
- [ ] Choose plain or ZK
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Both modes work
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Build an off-chain proof generation service

**Labels:** tooling, research, phase-4, difficulty: hard, help wanted

### Summary

Identity providers must generate proofs.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: cLI outputs a valid proof.

### Requirements

- [ ] Create a CLI and library
- [ ] Read attestations
- [ ] Output proofs

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] CLI outputs a valid proof
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Add Semaphore compatibility research

**Labels:** research, phase-4, difficulty: medium

### Summary

PRD lists Semaphore and zkPassport.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: findings doc merged.

### Requirements

- [ ] Evaluate group membership proofs for eligibility
- [ ] Write findings

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Findings doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Add zkPassport compatibility research

**Labels:** research, phase-4, difficulty: medium

### Summary

Same.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: findings doc merged.

### Requirements

- [ ] Evaluate passport proofs as KYC input
- [ ] Write findings

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Findings doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Benchmark verification latency against the 100 ms target

**Labels:** contracts, testing, phase-4, difficulty: medium

### Summary

Phase 4 exit criterion.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: report published.

### Requirements

- [ ] Measure simulated latency
- [ ] Report percentiles
- [ ] Optimize hot paths

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Report published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add ZK verifier to the SDK

**Labels:** sdk, phase-4, difficulty: medium

### Summary

SDK parity.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Add client
- [ ] Add a proof submission helper
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add the ZK threat model

**Labels:** documentation, security, phase-4, difficulty: hard, help wanted

### Summary

New attack surface.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: merged.

### Requirements

- [ ] Cover soundness bugs, key compromise and metadata leaks
- [ ] Store in `docs/threat-models/`

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Commission a cryptography review of the circuit

**Labels:** security, documentation, phase-4, difficulty: hard, help wanted

### Summary

Specialized review.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] Write scope and shortlist reviewers
- [ ] Set a timeline

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write the privacy design guide for issuers

**Labels:** documentation, phase-4, good first issue

### Summary

Issuers must choose settings.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Explain what is private and what leaks on-chain
- [ ] Add examples

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write ADR-013: cross-chain bridge architecture

**Labels:** documentation, contracts, security, phase-4, difficulty: hard, help wanted

### Summary

Bridges are the highest risk component.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Compare lock-and-mint with message-passing
- [ ] Define trust model, relayer bonding and dispute window

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Scaffold the bridge crate

**Labels:** contracts, phase-4, good first issue

### Summary

No crate exists.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement lock on Stellar

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Lock side.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: lock emits the event.

### Requirements

- [ ] Lock tokens
- [ ] Emit a message event with nonce
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Lock emits the event
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement unlock on Stellar with proof of burn

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Return side.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: unlock requires a valid attestation.

### Requirements

- [ ] Verify an attestation of burn
- [ ] Unlock tokens
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Unlock requires a valid attestation
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Define the cross-chain message format

**Labels:** contracts, documentation, phase-4, difficulty: medium

### Summary

Both sides must agree.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: test vectors pass on both sides.

### Requirements

- [ ] Specify fields, encoding and versioning
- [ ] Add test vectors

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Test vectors pass on both sides
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Implement relayer registration with economic bonds

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

PRD requires bonded relayers.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: unbonded relayer cannot attest.

### Requirements

- [ ] Add bond deposit
- [ ] Track relayer status
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Unbonded relayer cannot attest
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement threshold attestation by relayers

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

Prevent single relayer fraud.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: m minus 1 signatures fail.

### Requirements

- [ ] Require M of N signatures
- [ ] Verify signatures
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] M minus 1 signatures fail
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement the 7-day dispute window

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

PRD requires fraud proofs.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: disputed message is blocked.

### Requirements

- [ ] Delay finality
- [ ] Allow disputes with evidence
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Disputed message is blocked
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement relayer slashing

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

Penalty for fraud.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: slash flows reconcile.

### Requirements

- [ ] Slash bonds on proven fraud
- [ ] Pay challengers
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Slash flows reconcile
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add bridge rate limits and caps

**Labels:** contracts, security, phase-4, difficulty: medium

### Summary

Limits reduce loss in exploits.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: over-cap transfers are queued or rejected.

### Requirements

- [ ] Add per-window and per-asset caps
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Over-cap transfers are queued or rejected
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add a bridge emergency pause with guardian

**Labels:** contracts, security, phase-4, difficulty: medium

### Summary

Incident response.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: pause blocks lock and unlock.

### Requirements

- [ ] Add pause
- [ ] Define who can resume
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Pause blocks lock and unlock
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Gate bridge transfers through compliance

**Labels:** contracts, phase-4, difficulty: medium

### Summary

RWA transfers stay regulated across chains.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: non-compliant lock fails.

### Requirements

- [ ] Check compliance on lock
- [ ] Document receiving side rules

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Non-compliant lock fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Write the Ethereum ERC-20 wrapper contract

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

EVM side, separate repo per roadmap.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: foundry tests pass.

### Requirements

- [ ] Create the repo layout and Solidity contract
- [ ] Add mint and burn by the bridge
- [ ] Add Foundry tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Foundry tests pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add transfer restrictions to the ERC-20 wrapper

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Wrapped RWA tokens still need compliance.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: tests pass.

### Requirements

- [ ] Add an allowlist hook
- [ ] Document trust assumptions

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Tests pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Write the Ethereum bridge verifier contract

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

Verifies relayer attestations.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: tests pass.

### Requirements

- [ ] Implement threshold checks
- [ ] Add Foundry tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Tests pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Run the relayer node reference implementation

**Labels:** tooling, phase-4, difficulty: hard, help wanted

### Summary

Needed to operate the bridge.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: relays a testnet transfer.

### Requirements

- [ ] Watch Stellar events and submit to Ethereum
- [ ] Persist state
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Relays a testnet transfer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add relayer monitoring and slashing watcher

**Labels:** tooling, security, phase-4, difficulty: medium

### Summary

Watchers detect fraud.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: watcher flags a forged message in a test.

### Requirements

- [ ] Compare both chains
- [ ] Raise disputes
- [ ] Add alerts

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Watcher flags a forged message in a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Test 100 round-trip transfers on testnets

**Labels:** contracts, testing, phase-4, difficulty: hard, help wanted

### Summary

Phase 4 exit criterion.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: 100 successful round trips documented.

### Requirements

- [ ] Script round trips between Stellar testnet and Sepolia
- [ ] Record failures
- [ ] Publish results

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] 100 successful round trips documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Fuzz bridge message handling

**Labels:** contracts, testing, security, phase-4, difficulty: hard, help wanted

### Summary

Parsing bugs are fatal.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: no panics found.

### Requirements

- [ ] Fuzz decoding and validation
- [ ] Run nightly

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] No panics found
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add the bridge to the SDK

**Labels:** sdk, phase-4, difficulty: medium

### Summary

SDK parity.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Add lock and status helpers
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a bridge status page

**Labels:** frontend, phase-4, difficulty: medium

### Summary

Users track transfers.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: page shows a test transfer.

### Requirements

- [ ] Show state, confirmations and dispute window

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Page shows a test transfer
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Write the bridge threat model

**Labels:** documentation, security, phase-4, difficulty: hard, help wanted

### Summary

Mandatory.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: merged.

### Requirements

- [ ] Cover relayer collusion, replay and chain reorgs
- [ ] Store in `docs/threat-models/`

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Commission a dedicated bridge audit

**Labels:** security, documentation, phase-4, difficulty: hard, help wanted

### Summary

Beyond the Phase 4 audit.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] Write scope and shortlist
- [ ] Set timeline

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write ADR-014: Cosmos IBC integration

**Labels:** documentation, research, phase-4, difficulty: hard, help wanted

### Summary

Soroban has no native IBC.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Evaluate light client approaches
- [ ] Decide on interchain accounts
- [ ] Document risks

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype an IBC light client verifier in Soroban

**Labels:** research, contracts, phase-4, difficulty: hard, help wanted

### Summary

Feasibility.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: prototype verifies a header.

### Requirements

- [ ] Verify a Tendermint header
- [ ] Measure cost
- [ ] Document

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Prototype verifies a header
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Define the IBC channel handshake handling

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Channel lifecycle.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: handshake test passes.

### Requirements

- [ ] Implement connection and channel state machines
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Handshake test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the Stellar to Cosmos transfer packet

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Core flow.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: packet round trip works against a local chain.

### Requirements

- [ ] Define the packet
- [ ] Send and acknowledge
- [ ] Add timeouts

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Packet round trip works against a local chain
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement packet timeout and refund

**Labels:** contracts, phase-4, difficulty: medium

### Summary

Failure path.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: timed out packet refunds.

### Requirements

- [ ] Refund on timeout
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Timed out packet refunds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add interchain account support for multi-step settlement

**Labels:** contracts, research, phase-4, difficulty: hard, help wanted

### Summary

PRD lists it.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: prototype executes one remote call.

### Requirements

- [ ] Design the account model
- [ ] Prototype
- [ ] Document

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Prototype executes one remote call
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Run a local Cosmos test chain in CI

**Labels:** ci, testing, phase-4, difficulty: medium

### Summary

Integration tests need a chain.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: cI job starts the chain.

### Requirements

- [ ] Use a containerized chain
- [ ] Cache images

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] CI job starts the chain
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- `contracts/*/tests/`
- `sdk/tests/`

---

## Write the Cosmos side contract or module

**Labels:** contracts, phase-4, difficulty: hard, help wanted

### Summary

Receiving side.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: tests pass.

### Requirements

- [ ] Implement a CosmWasm contract for wrapped tokens
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Tests pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Document the cross-chain user flow

**Labels:** documentation, phase-4, good first issue

### Summary

Users need a guide.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Explain lock, wait, claim and failure cases

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Research Pedersen commitment balances

**Labels:** research, phase-4, difficulty: hard, help wanted

### Summary

Roadmap marks privacy transfers as a research track.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Survey designs
- [ ] Estimate Soroban costs
- [ ] Write recommendation

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype a confidential transfer with range proofs

**Labels:** research, phase-4, difficulty: hard, help wanted

### Summary

Feasibility study.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: prototype and report merged.

### Requirements

- [ ] Implement a minimal prototype off-chain
- [ ] Estimate verification cost on-chain

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Prototype and report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Define the guardian retirement plan

**Labels:** documentation, security, phase-4, difficulty: medium

### Summary

Phase 4 retires the multisig guardian.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] List every guardian power
- [ ] Map each to governance
- [ ] Order the steps

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Move rwa-asset admin to governance on testnet

**Labels:** contracts, testing, phase-4, difficulty: medium

### Summary

Proof of the migration.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: evidence published.

### Requirements

- [ ] Use dual-auth rotation
- [ ] Execute a parameter change by proposal
- [ ] Record

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Evidence published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Move registry admin to governance on testnet

**Labels:** contracts, testing, phase-4, good first issue

### Summary

Same.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: evidence published.

### Requirements

- [ ] Rotate admin
- [ ] Run a delist via proposal

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Evidence published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Move compliance admin to governance with delegated operators

**Labels:** contracts, security, phase-4, difficulty: hard, help wanted

### Summary

Compliance needs day-to-day operators.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: operators act, governance rotates.

### Requirements

- [ ] Split governance and operator roles
- [ ] Add tests

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Operators act, governance rotates
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Move oracle-adapter admin to governance

**Labels:** contracts, testing, phase-4, good first issue

### Summary

Same.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: evidence published.

### Requirements

- [ ] Rotate admin
- [ ] Run a config change by proposal

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Evidence published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add an admin key inventory and verification script

**Labels:** tooling, security, phase-4, difficulty: medium

### Summary

Exit criteria require no multisig admin keys.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: script outputs a clean report on testnet.

### Requirements

- [ ] Query every contract admin
- [ ] Report non-governance admins

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Script outputs a clean report on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write the Phase 4 external audit scope

**Labels:** documentation, security, phase-4, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] List contracts and commits
- [ ] Include ZK and bridge
- [ ] Estimate effort

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Prepare the Phase 3 mainnet deployment plan

**Labels:** documentation, phase-4, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 4 (target Q3 2027) adds ZK compliance proofs with an on-chain verifier, a Stellar to Ethereum lock-and-mint bridge, a Cosmos IBC bridge and retirement of the multisig guardian. See ROADMAP.md Phase 4 and PRD sections 5.8, 5.9, 12 and 14.

This issue belongs to **Stage 7: Phase 4 - Privacy and Cross-Chain**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Define order and checks
- [ ] Reuse prior checklists

### Out of Scope

Insurance, treasury and the full DAO migration (Phase 5).

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting -->

## Write ADR-015: insurance pool economics

**Labels:** documentation, contracts, research, phase-5, difficulty: hard, help wanted

### Summary

Premium pricing and solvency rules decide whether the pool survives.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: aDR merged with the model.

### Requirements

- [ ] Model premiums, collateral and payout scenarios
- [ ] Set the 150 percent minimum ratio rule
- [ ] Record assumptions

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] ADR merged with the model
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Scaffold the insurance-pool crate

**Labels:** contracts, phase-5, good first issue

### Summary

No crate exists.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement collateral staking by insurance providers

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Backing for policies.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: providers stake and withdraw per rules.

### Requirements

- [ ] Accept deposits
- [ ] Track shares of the pool
- [ ] Add lock rules

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Providers stake and withdraw per rules
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement policy purchase with premium payment

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Permissionless coverage.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: policy is stored and premium split correctly.

### Requirements

- [ ] Accept coverage amount and duration
- [ ] Compute premium
- [ ] Mint a policy record

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Policy is stored and premium split correctly
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement premium pricing curves

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Price reflects risk and utilization.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: price rises with utilization.

### Requirements

- [ ] Implement a utilization-based curve
- [ ] Add parameters
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Price rises with utilization
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Enforce the minimum coverage solvency ratio

**Labels:** contracts, security, phase-5, difficulty: hard, help wanted

### Summary

PRD requires at least 150 percent.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: boundary test passes.

### Requirements

- [ ] Reject policies that break the ratio
- [ ] Add tests at the boundary

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Boundary test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement coverage scope definitions

**Labels:** contracts, phase-5, difficulty: medium

### Summary

Bugs, oracle manipulation and bridge exploits are covered.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: out-of-scope claims fail.

### Requirements

- [ ] Encode scopes per policy
- [ ] Validate on claim
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Out-of-scope claims fail
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement claim submission with evidence hash

**Labels:** contracts, phase-5, difficulty: medium

### Summary

Claims need evidence.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: claim is stored.

### Requirements

- [ ] Accept claim data and evidence hash
- [ ] Bond the claimant
- [ ] Emit events

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Claim is stored
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement DAO adjudication of claims

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Claims decided by vote.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: approved claim pays out.

### Requirements

- [ ] Create a governance proposal per claim
- [ ] Pay on approval
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Approved claim pays out
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement payout and pro-rata loss sharing

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Pool losses are shared.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: losses reconcile.

### Requirements

- [ ] Reduce provider shares on payout
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Losses reconcile
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement policy expiry and premium earning

**Labels:** contracts, phase-5, difficulty: medium

### Summary

Premiums accrue over time.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: accrual matches the schedule.

### Requirements

- [ ] Release premiums linearly
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Accrual matches the schedule
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add insurance pool pause and wind-down

**Labels:** contracts, security, phase-5, difficulty: medium

### Summary

Failure handling.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: providers withdraw after wind-down.

### Requirements

- [ ] Add pause and wind-down
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Providers withdraw after wind-down
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add insurance events and snapshot tests

**Labels:** contracts, testing, phase-5, good first issue

### Summary

IR-03 practice.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: tests exist.

### Requirements

- [ ] Define events
- [ ] Snapshot
- [ ] Document

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Tests exist
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add admin rotation and constructor auth tests to insurance-pool

**Labels:** contracts, testing, security, phase-5, good first issue

### Summary

IR-04 and IR-17.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: tests mutation-checked.

### Requirements

- [ ] Add `transfer_admin` and the auth test

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Tests mutation-checked
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Property test insurance solvency

**Labels:** contracts, testing, phase-5, difficulty: hard, help wanted

### Summary

The pool must never go insolvent silently.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: no violation in 100,000 cases.

### Requirements

- [ ] Randomize policies, claims and withdrawals
- [ ] Assert the ratio rule

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] No violation in 100,000 cases
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Run a 90-day solvency simulation

**Labels:** research, testing, phase-5, difficulty: hard, help wanted

### Summary

Phase 5 exit criterion.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: report shows ratio above 200 percent.

### Requirements

- [ ] Build an agent simulation
- [ ] Report ratio over time

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Report shows ratio above 200 percent
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add InsurancePoolClient to the SDK

**Labels:** sdk, phase-5, difficulty: medium

### Summary

SDK parity.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Wrap functions
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add an insurance dashboard page

**Labels:** frontend, phase-5, difficulty: medium

### Summary

Users buy coverage.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: purchase works on testnet.

### Requirements

- [ ] Quote, buy and view policies

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Purchase works on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Write the insurance pool threat model

**Labels:** documentation, security, phase-5, difficulty: hard, help wanted

### Summary

Mandatory.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: merged.

### Requirements

- [ ] Cover adjudication capture, oracle abuse and bank-run dynamics

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write the insurance disclaimer and risk copy

**Labels:** documentation, phase-5, good first issue

### Summary

Avoid misleading users.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: copy merged.

### Requirements

- [ ] Draft plain-language risk text
- [ ] Review with legal

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Copy merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write ADR-016: optimistic governance and veto council

**Labels:** documentation, contracts, security, phase-5, difficulty: hard, help wanted

### Summary

Phase 5 uses a 48-hour timelock with a veto.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Define flow, veto rules and council election

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement optimistic execution with a 48h timelock

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Proposals execute unless vetoed.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: unvetoed proposal executes after 48h.

### Requirements

- [ ] Add optimistic path
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Unvetoed proposal executes after 48h
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the 3-of-7 emergency veto council

**Labels:** contracts, security, phase-5, difficulty: hard, help wanted

### Summary

PRD defines it.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: three vetoes block execution.

### Requirements

- [ ] Add council membership and veto votes
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Three vetoes block execution
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement council election and rotation

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Elected community members.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: council changes by election.

### Requirements

- [ ] Add elections by SFORGE vote
- [ ] Add terms
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Council changes by election
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement vote delegation limits and anti-capture checks

**Labels:** contracts, security, phase-5, difficulty: medium

### Summary

Prevent concentration.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: cap enforced.

### Requirements

- [ ] Add per-delegate caps or warnings
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Cap enforced
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add proposal categories with different thresholds

**Labels:** contracts, phase-5, difficulty: medium

### Summary

Parameter changes differ from upgrades.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: upgrades need higher quorum.

### Requirements

- [ ] Add categories and quorum per category
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Upgrades need higher quorum
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add on-chain proposal metadata hash for off-chain discussion

**Labels:** contracts, phase-5, good first issue

### Summary

PRD 5.7 plans this.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: hash retrievable.

### Requirements

- [ ] Store a metadata hash
- [ ] Emit it
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Hash retrievable
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Migrate all admin keys to governance

**Labels:** contracts, security, phase-5, difficulty: hard, help wanted

### Summary

Exit criterion.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: verification script clean.

### Requirements

- [ ] Run the rotation across all contracts on testnet
- [ ] Verify no admin keys remain

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Verification script clean
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Relinquish upgrade authority behind a governance-only path

**Labels:** contracts, security, phase-5, difficulty: hard, help wanted

### Summary

No admin keys.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: upgrade proposal executes on testnet.

### Requirements

- [ ] Move the upgrade right to governance
- [ ] Test an upgrade by proposal

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Upgrade proposal executes on testnet
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Approve and execute a fee change by DAO vote on testnet

**Labels:** contracts, testing, phase-5, difficulty: medium

### Summary

Exit criterion.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: evidence published.

### Requirements

- [ ] Run the full flow
- [ ] Record hashes

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Evidence published
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Write ADR-017: protocol treasury design

**Labels:** documentation, contracts, phase-5, difficulty: medium

### Summary

Defines holdings and spending rules.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Specify accounts, spend limits and reporting

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the treasury crate

**Labels:** contracts, phase-5, good first issue

### Summary

No crate exists.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement fee accumulation from protocol modules

**Labels:** contracts, phase-5, difficulty: medium

### Summary

Fees flow to the treasury.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: balances reconcile.

### Requirements

- [ ] Accept fees from authorized modules
- [ ] Track per token
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Balances reconcile
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement DAO-controlled spending with limits

**Labels:** contracts, security, phase-5, difficulty: hard, help wanted

### Summary

Spending needs guardrails.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: over-cap spend fails.

### Requirements

- [ ] Add per-proposal and per-period caps
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Over-cap spend fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Implement the grant program module

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

PRD lists ecosystem grants.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: grant releases per milestone.

### Requirements

- [ ] Add milestones and clawback
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Grant releases per milestone
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement buyback-and-burn for SFORGE

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

PRD lists it.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: burn event emitted in tests.

### Requirements

- [ ] Swap through a DEX adapter
- [ ] Burn tokens
- [ ] Add slippage limits

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Burn event emitted in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add treasury reporting views

**Labels:** contracts, phase-5, good first issue

### Summary

Transparency.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: views match state.

### Requirements

- [ ] Add balance and flow views
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Views match state
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add TreasuryClient to the SDK

**Labels:** sdk, phase-5, difficulty: medium

### Summary

SDK parity.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: covers the ABI.

### Requirements

- [ ] Wrap functions
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Covers the ABI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Add a public treasury dashboard

**Labels:** frontend, phase-5, difficulty: medium

### Summary

Transparency.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: page renders testnet data.

### Requirements

- [ ] Show balances, flows and grants

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Page renders testnet data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The UI works on testnet, is keyboard accessible and has no console errors
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)

---

## Write ADR-018: institutional custody adapter interface

**Labels:** documentation, contracts, phase-5, difficulty: medium

### Summary

Fireblocks, Copper and BitGo differ.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Define a common signing and policy interface
- [ ] Document differences

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement a Fireblocks signer adapter

**Labels:** sdk, tooling, phase-5, difficulty: hard, help wanted

### Summary

Enterprise issuers.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: signs a testnet transaction.

### Requirements

- [ ] Use the Fireblocks API to sign Soroban transactions
- [ ] Add mocks and tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Signs a testnet transaction
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Implement a Copper signer adapter

**Labels:** sdk, tooling, phase-5, difficulty: hard, help wanted

### Summary

Same.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: signs a testnet transaction.

### Requirements

- [ ] Implement against the Copper API
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Signs a testnet transaction
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Implement a BitGo signer adapter

**Labels:** sdk, tooling, phase-5, difficulty: hard, help wanted

### Summary

Same.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: signs a testnet transaction.

### Requirements

- [ ] Implement against the BitGo API
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Signs a testnet transaction
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add custody policy templates for issuers

**Labels:** documentation, phase-5, good first issue

### Summary

Safe defaults.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: templates merged.

### Requirements

- [ ] Provide policy examples for mint approvals
- [ ] Document

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Templates merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write ADR-019: regulatory reporting data model

**Labels:** documentation, contracts, phase-5, difficulty: medium

### Summary

MiCA, SEC Reg D/A+ and MAS differ.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: aDR merged.

### Requirements

- [ ] Map required fields to on-chain data
- [ ] Identify gaps

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] ADR merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Scaffold the reporting module crate

**Labels:** contracts, phase-5, good first issue

### Summary

No crate exists.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: crate builds.

### Requirements

- [ ] Create crate files
- [ ] Add to workspace

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Crate builds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement the on-chain issuance ledger

**Labels:** contracts, phase-5, difficulty: hard, help wanted

### Summary

Canonical issuance record.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: ledger entries are append-only.

### Requirements

- [ ] Record issuance and redemption entries
- [ ] Hash entries
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Ledger entries are append-only
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Implement a holder snapshot export

**Labels:** contracts, tooling, phase-5, difficulty: medium

### Summary

Reports need holder lists.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: export matches balances.

### Requirements

- [ ] Provide a bounded paged view and an off-chain exporter
- [ ] Add tests

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Export matches balances
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `scripts/`
- [Makefile](Makefile)

---

## Implement MiCA report generator (off-chain)

**Labels:** tooling, phase-5, difficulty: hard, help wanted

### Summary

Regulatory output.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: sample report generated.

### Requirements

- [ ] Generate the required fields from chain data
- [ ] Add sample output

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Sample report generated
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Implement SEC Reg D and A+ report generator

**Labels:** tooling, phase-5, difficulty: hard, help wanted

### Summary

Same.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: sample report generated.

### Requirements

- [ ] Generate required data
- [ ] Add sample

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Sample report generated
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Implement MAS report generator

**Labels:** tooling, phase-5, difficulty: hard, help wanted

### Summary

Same.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: sample report generated.

### Requirements

- [ ] Generate required data
- [ ] Add sample

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Sample report generated
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add report signing and timestamping

**Labels:** tooling, security, phase-5, difficulty: medium

### Summary

Reports need integrity.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: signature verifies.

### Requirements

- [ ] Sign output with a configured key
- [ ] Include ledger references

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Signature verifies
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Write the Phase 5 external audit scope

**Labels:** documentation, security, phase-5, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: scope doc merged.

### Requirements

- [ ] List contracts and commits
- [ ] Estimate effort

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Scope doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Prepare the Phase 4 mainnet deployment plan

**Labels:** documentation, phase-5, difficulty: medium

### Summary

ROADMAP lists it.

### Context

Phase 5 (target Q1-Q2 2028) delivers the `insurance-pool` contract, full DAO migration with optimistic governance, a protocol treasury, institutional custody adapters and a regulatory reporting module. See ROADMAP.md Phase 5 and PRD sections 5.10, 11.5 and 13.3.

This issue belongs to **Stage 8: Phase 5 - Full DAO, Insurance, Treasury and Reporting**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Define order and checks

### Out of Scope

Research tracks listed under Beyond Phase 5.

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts) -->

## Add negative-path tests for rwa-asset mint

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Only happy paths and a few errors may be covered.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: each failure has a test.

### Requirements

- [ ] Test mint by non-issuer, over cap, negative and zero amount, paused
- [ ] Assert exact error variants

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Each failure has a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add negative-path tests for rwa-asset burn and burn_from

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Same.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: each failure has a test.

### Requirements

- [ ] Test insufficient balance, no allowance, expired allowance, paused

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Each failure has a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add negative-path tests for rwa-asset transfer

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Same.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: each failure has a test.

### Requirements

- [ ] Test zero, negative, insufficient funds, paused, failed compliance

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Each failure has a test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add boundary tests for allowance live_until_ledger

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Edge ledgers cause off-by-one bugs.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: boundaries documented in tests.

### Requirements

- [ ] Test expiry at ledger N minus 1, N and N plus 1
- [ ] Test live_until in the past

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Boundaries documented in tests
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add tests for self-transfer behavior

**Labels:** contracts, testing, phase-2, good first issue

### Summary

IR-15 touched self `transfer_from`.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior is pinned.

### Requirements

- [ ] Test self transfer and self transfer_from events and balances

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior is pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add tests for maximum i128 amounts

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Overflow is a classic bug.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: no panic reaches the host.

### Requirements

- [ ] Test balances near i128 max
- [ ] Assert overflow errors not panics

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] No panic reaches the host
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add compliance tests for exact expiry timestamps

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Expiry semantics can drift.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: boundaries pinned.

### Requirements

- [ ] Test at expiry minus 1, equal, plus 1

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Boundaries pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add compliance tests for revoke then re-set

**Labels:** contracts, testing, phase-2, good first issue

### Summary

State flows.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test revoke, set again and read
- [ ] Assert events

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add compliance tests for jurisdiction validation edge cases

**Labels:** contracts, testing, phase-2, good first issue

### Summary

IR-08 added checks.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: all rejected with `InvalidJurisdiction`.

### Requirements

- [ ] Test lowercase, three letters, digits, empty

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] All rejected with `InvalidJurisdiction`
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add registry tests for pagination boundaries

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Pagination bugs hide at boundaries.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test start beyond count, limit 0, limit 100, limit 101

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add registry tests for duplicate registration

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Duplicates must fail.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: error asserted.

### Requirements

- [ ] Test registering the same contract twice

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Error asserted
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add registry tests for set_active transitions

**Labels:** contracts, testing, phase-2, good first issue

### Summary

State flow.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test active to inactive to active and events

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance tests for exact voting period boundaries

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Boundary bugs.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test vote at last ledger and first ledger after
- [ ] Test finalize timing

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance tests for tie votes

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Tie handling must be explicit.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior documented.

### Requirements

- [ ] Test equal for and against
- [ ] Assert the outcome

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance tests for double voting

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Core safety property.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: second vote fails.

### Requirements

- [ ] Test same voter twice with different support

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Second vote fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance tests for finalize called twice

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Idempotence.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: second call fails.

### Requirements

- [ ] Test double finalize

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Second call fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add governance tests for zero weight votes

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Edge case.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: rejected.

### Requirements

- [ ] Test weight 0 and negative weights

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Rejected
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add oracle-adapter tests for price at exact timestamps

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Boundary behavior.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test `price` for exact, before and after known timestamps

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add oracle-adapter tests for the deviation guard boundaries

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Guard must trigger exactly at the limit.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test at limit, limit plus 1 bps

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add oracle-adapter tests for reporter removal

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Access control.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: report fails.

### Requirements

- [ ] Remove a reporter then report

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Report fails
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add oracle-adapter tests for the `prices` records limit

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Bounds.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Test records 0, 1 and above the cap

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add oracle-adapter override_price audit event tests

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Overrides are sensitive.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: test exists.

### Requirements

- [ ] Assert an event is emitted with the old and new values

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Test exists
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test cross-contract: rwa-asset with registry

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Registry lists assets.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: test passes.

### Requirements

- [ ] Register a deployed asset
- [ ] Deactivate
- [ ] Assert independent behavior

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test cross-contract: rwa-asset with compliance across expiry

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Real flow.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior pinned.

### Requirements

- [ ] Transfer before and after KYC expiry

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior pinned
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test cross-contract: governance controlling an rwa-asset admin

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Preparation for execution hooks.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: behavior documented.

### Requirements

- [ ] Rotate the admin to governance
- [ ] Try admin calls by proposal

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Behavior documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Test deployment order and constructor argument validation

**Labels:** contracts, testing, phase-2, good first issue

### Summary

Constructors validate inputs.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: each rejection tested.

### Requirements

- [ ] Test invalid constructor args for all crates

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Each rejection tested
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add a shared test utility crate

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Test setup is duplicated.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: duplication removed in migrated crates.

### Requirements

- [ ] Create `contracts/test-utils` with env builders
- [ ] Migrate two crates

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Duplication removed in migrated crates
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add snapshot tests for contract spec output

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

ABI changes should be intentional.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: aBI changes fail the test.

### Requirements

- [ ] Snapshot the exported spec per crate
- [ ] Document updates

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] ABI changes fail the test
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add code coverage reporting for contracts

**Labels:** contracts, ci, testing, phase-2, difficulty: medium

### Summary

No coverage metrics.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: coverage appears on PRs.

### Requirements

- [ ] Use cargo-llvm-cov
- [ ] Upload to Codecov
- [ ] Report per crate

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Coverage appears on PRs
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`
- `contracts/*/tests/`
- `sdk/tests/`

---

## Set a coverage threshold for contracts

**Labels:** contracts, ci, phase-2, good first issue

### Summary

Needs a number.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: pR fails below threshold.

### Requirements

- [ ] Set 90 percent line threshold
- [ ] Document exceptions

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] PR fails below threshold
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Add mutation testing for contracts with cargo-mutants

**Labels:** contracts, testing, security, phase-2, difficulty: hard, help wanted

### Summary

Coverage does not prove assertions.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: score reported.

### Requirements

- [ ] Run on rwa-asset and compliance
- [ ] Kill surviving mutants
- [ ] Report

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Score reported
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Review clippy allow attributes and remove unjustified ones

**Labels:** contracts, phase-2, good first issue

### Summary

Suppressed lints hide issues.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: each remaining allow has a reason.

### Requirements

- [ ] List all `allow` attributes
- [ ] Remove or justify with comments

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Each remaining allow has a reason
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Enable stricter clippy lints in the workspace

**Labels:** contracts, ci, phase-2, difficulty: medium

### Summary

Higher quality bar.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: cI passes.

### Requirements

- [ ] Enable `clippy::pedantic` selectively
- [ ] Fix findings
- [ ] Document choices

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] CI passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `.github/workflows/`

---

## Replace magic numbers with named constants

**Labels:** contracts, phase-2, good first issue

### Summary

Constants such as TTL values should be named.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: no unexplained literals remain.

### Requirements

- [ ] Find literals in `lib.rs` files
- [ ] Move to `common`
- [ ] Add docs

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] No unexplained literals remain
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add rustdoc to every public contract function

**Labels:** contracts, documentation, phase-2, good first issue

### Summary

Docs generate the spec descriptions.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: warning-free doc build.

### Requirements

- [ ] Document args, auth, errors and events
- [ ] Enable `missing_docs`

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Warning-free doc build
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Publish rustdoc to GitHub Pages

**Labels:** documentation, ci, phase-2, good first issue

### Summary

Make docs browsable.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: site is live.

### Requirements

- [ ] Build docs in CI
- [ ] Deploy to Pages

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Site is live
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `.github/workflows/`

---

## Factor shared admin logic into contracts/common

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Admin rotation is duplicated across crates.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: no behavior change.

### Requirements

- [ ] Extract a helper preserving dual authorization
- [ ] Migrate crates one at a time
- [ ] Keep tests green

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] No behavior change
- [ ] WASM size delta reported
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Factor shared pagination into contracts/common

**Labels:** contracts, phase-2, good first issue

### Summary

Pagination logic appears in the registry.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: registry tests unchanged.

### Requirements

- [ ] Extract helper
- [ ] Reuse in governance listing

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Registry tests unchanged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Review event naming consistency across contracts

**Labels:** contracts, phase-2, good first issue

### Summary

Names should follow one scheme.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: convention doc merged.

### Requirements

- [ ] Audit event names
- [ ] Propose a convention
- [ ] Apply if no consumers break

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Convention doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Review error enum numbering consistency

**Labels:** contracts, phase-2, good first issue

### Summary

Codes should not collide in meaning across crates.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: table merged.

### Requirements

- [ ] Audit codes
- [ ] Document ranges per crate

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Table merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Optimize storage reads in transfer

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Fewer reads lower fees.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: instruction count drops without behavior change.

### Requirements

- [ ] Profile transfer
- [ ] Remove redundant reads
- [ ] Show before and after numbers

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Instruction count drops without behavior change
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Optimize registry registration cost

**Labels:** contracts, phase-2, difficulty: medium

### Summary

Same.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: cost drops.

### Requirements

- [ ] Profile register
- [ ] Remove redundant writes

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Cost drops
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Add a security-focused review of all require_auth placements

**Labels:** contracts, security, phase-2, difficulty: medium

### Summary

Auth bugs are critical.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: table merged.

### Requirements

- [ ] Review each entry point
- [ ] Record the auth table in the docs
- [ ] Add missing tests

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Table merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

## Add an inventory of all external calls made by contracts

**Labels:** contracts, security, phase-2, good first issue

### Summary

Reentrancy analysis input.

### Context

The five existing contracts have integration tests per crate but gaps remain in negative paths, boundaries and documentation. This stage is a good entry point for new contributors.

This issue belongs to **Stage 9: Contract Test Depth and Code Quality (Phase 1 Contracts)**.

### Objective

Deliver the change described above so that: inventory merged.

### Requirements

- [ ] List cross-contract calls and order relative to state writes

### Out of Scope

New features. Tests and refactors must not change contract behavior.

### Acceptance Criteria

- [ ] Inventory merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)

---

<!-- Stage 10: Research Tracks, Ecosystem and Community -->

## Research sub-second NAV updates via the Stellar DEX

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Real-time NAV settlement is a listed track.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Survey DEX price discovery
- [ ] Estimate manipulation risk
- [ ] Write findings

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype oracle aggregation across DEX and external feeds

**Labels:** research, contracts, research, difficulty: hard, help wanted

### Summary

Same track.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: prototype and notes merged.

### Requirements

- [ ] Build an aggregator prototype
- [ ] Test on testnet data

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Prototype and notes merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Design a settlement mechanism using live NAV

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Write a design doc with failure modes

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Research tokenized private credit structures

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Tokenized debt is a listed track.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Survey structures
- [ ] List covenants to monitor

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Design on-chain covenant monitoring

**Labels:** research, contracts, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: spec merged.

### Requirements

- [ ] Specify covenants and triggers
- [ ] Draft interfaces

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Spec merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Prototype a loan contract with repayment schedule

**Labels:** research, contracts, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: prototype merged.

### Requirements

- [ ] Implement a minimal loan with schedule
- [ ] Add tests

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Prototype merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Design default and recovery handling for tokenized debt

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Write the design with tranche links

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Research carbon credit tokenization standards

**Labels:** research, research, difficulty: medium

### Summary

Verra and Gold Standard are listed.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Survey registry rules and retirement
- [ ] Write findings

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype a carbon credit asset type

**Labels:** research, contracts, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: prototype merged.

### Requirements

- [ ] Add retirement semantics to a prototype asset
- [ ] Add tests

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Prototype merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Design registry bridges to Verra and Gold Standard serial numbers

**Labels:** research, documentation, research, difficulty: medium

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Specify metadata mapping
- [ ] Document double counting risks

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Research AI-assisted due diligence attestations

**Labels:** research, research, difficulty: medium

### Summary

Listed track.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Define attestation format and trust model

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype an on-chain attestation registry

**Labels:** research, contracts, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: prototype merged.

### Requirements

- [ ] Implement attestations signed by approved underwriters
- [ ] Add tests

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Prototype merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)

---

## Research recursive ZK rollups for batching RWA operations

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Listed track.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Survey proof systems and Soroban constraints

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Prototype batch transfer validity proofs

**Labels:** research, research, difficulty: hard, help wanted

### Summary

Same.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: prototype merged.

### Requirements

- [ ] Prototype an off-chain prover for a batch of transfers

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Prototype merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Run a developer survey on RWA tokenization needs

**Labels:** documentation, research, good first issue

### Summary

Roadmap input.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: summary merged.

### Requirements

- [ ] Draft questions
- [ ] Publish
- [ ] Summarize results

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Summary merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Create a public roadmap board with phase columns

**Labels:** documentation, tooling, research, good first issue

### Summary

Roadmap visibility.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: board is live.

### Requirements

- [ ] Create a GitHub Project
- [ ] Add issues by phase label
- [ ] Link from README

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Board is live
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `scripts/`
- [Makefile](Makefile)

---

## Add an examples repository index

**Labels:** documentation, research, good first issue

### Summary

Examples are scattered.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: index merged.

### Requirements

- [ ] List all examples in `docs/EXAMPLES.md`

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Index merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Create a contributor onboarding video script

**Labels:** documentation, research, good first issue

### Summary

Lowers entry cost.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: script merged.

### Requirements

- [ ] Write a 10 minute script covering setup and a first PR

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Script merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Create a first-contributor tutorial: fix a labeled issue

**Labels:** documentation, research, good first issue

### Summary

Hands-on guide.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: tutorial merged.

### Requirements

- [ ] Walk through fork, branch, test and PR with screenshots

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Tutorial merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a maintainers guide

**Labels:** documentation, research, good first issue

### Summary

Maintenance practices.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Document triage, review SLAs and release duty

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a governance of the project document (non-protocol)

**Labels:** documentation, research, good first issue

### Summary

Decision making for contributors.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: document merged.

### Requirements

- [ ] Describe maintainers, voting on ADRs and conflict resolution

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Document merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a contributor recognition automation

**Labels:** documentation, ci, research, good first issue

### Summary

CONTRIBUTING lists recognition.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: bot works on a test PR.

### Requirements

- [ ] Add an all-contributors config and bot

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Bot works on a test PR
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `.github/workflows/`

---

## Create a discussion category structure

**Labels:** documentation, research, good first issue

### Summary

Discussions are mentioned in the roadmap.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: categories created.

### Requirements

- [ ] Define categories for ideas, Q&A and RFCs

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Categories created
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add an RFC process and template

**Labels:** documentation, research, good first issue

### Summary

Large changes need a proposal format.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: template merged.

### Requirements

- [ ] Create `docs/rfcs/0000-template.md`
- [ ] Describe the flow

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Template merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write a comparison with other RWA token standards

**Labels:** documentation, research, research, difficulty: medium

### Summary

Positioning.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Compare ERC-3643, ERC-1400 and Stellar assets
- [ ] List tradeoffs

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Map StellarForge to ERC-3643 concepts

**Labels:** documentation, research, difficulty: medium

### Summary

Eases migration for EVM teams.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: table merged.

### Requirements

- [ ] Create a concept table

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Table merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Write a guide for migrating from classic Stellar assets to Soroban RWA tokens

**Labels:** documentation, research, difficulty: medium

### Summary

Common user need.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: guide merged.

### Requirements

- [ ] Explain wrapping with SACs and trustline differences

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Guide merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Evaluate integration with Stellar anchors (SEP-24, SEP-31)

**Labels:** research, research, difficulty: medium

### Summary

Fiat on and off ramps.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Describe flows for subscription and redemption

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)

---

## Evaluate wallet compatibility (Freighter, Lobstr, xBull)

**Labels:** research, testing, research, difficulty: medium

### Summary

Real wallet support.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: matrix merged.

### Requirements

- [ ] Test token display and transfer in each wallet
- [ ] Record results

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Matrix merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Findings are written up in `docs/` with sources, tradeoffs and a recommendation
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [ROADMAP.md](ROADMAP.md)
- [PRD](docs/prd/PRODUCT_REQUIREMENTS.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Create a public testnet faucet script for demo assets

**Labels:** tooling, research, good first issue

### Summary

Demos need test funds.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: script works.

### Requirements

- [ ] Add a script that mints demo assets to an address with limits

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Script works
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Create a demo seed script for the dashboard

**Labels:** tooling, research, good first issue

### Summary

Demos need data.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: dashboard shows data.

### Requirements

- [ ] Seed assets, KYC records and proposals on testnet

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Dashboard shows data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add brand and logo guidelines

**Labels:** documentation, research, good first issue

### Summary

Consistency.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: merged.

### Requirements

- [ ] Add `docs/BRAND.md` and assets

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Publish a public changelog site

**Labels:** documentation, research, good first issue

### Summary

Release visibility.

### Context

ROADMAP.md lists research tracks beyond Phase 5 and a contributor program. This stage turns them into scoped, first-step issues and builds the community around the protocol.

This issue belongs to **Stage 10: Research Tracks, Ecosystem and Community**.

### Objective

Deliver the change described above so that: site live.

### Requirements

- [ ] Generate from CHANGELOG.md
- [ ] Host on Pages

### Out of Scope

Committing the protocol to any track. Outputs are reports and prototypes.

### Acceptance Criteria

- [ ] Site live
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

<!-- Stage 11: Operations, Observability and Release Engineering -->

## Add a nightly testnet integration workflow for all contracts

**Labels:** ci, testing, phase-2, difficulty: medium

### Summary

`testnet-smoke.yml` exists, but coverage should include every deployed contract.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: a failing night opens an issue.

### Requirements

- [ ] Deploy fresh contracts nightly
- [ ] Run read and write smoke tests
- [ ] Post failures to an issue

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] A failing night opens an issue
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- `contracts/*/tests/`
- `sdk/tests/`

---

## Cache the Rust build and WASM artifacts across CI jobs

**Labels:** ci, phase-2, good first issue

### Summary

Slow CI discourages contributors.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: cI time drops by a measured amount.

### Requirements

- [ ] Tune cache keys
- [ ] Share artifacts between jobs
- [ ] Report timing

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] CI time drops by a measured amount
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Add a matrix test for Node 20 and 22 in SDK CI

**Labels:** sdk, ci, phase-2, good first issue

### Summary

Supported Node versions need coverage.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: both pass.

### Requirements

- [ ] Add a matrix
- [ ] Document supported versions

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Both pass
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `.github/workflows/`

---

## Add a release checklist issue template

**Labels:** ci, documentation, phase-2, good first issue

### Summary

Releases are error prone.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: template available.

### Requirements

- [ ] Add a template with signing, hashes and changelog steps

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Template available
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Automate GitHub Releases with WASM artifacts and hashes

**Labels:** ci, phase-2, difficulty: medium

### Summary

Releases ship artifacts manually.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: tag creates a release.

### Requirements

- [ ] Upload WASM and `SHA256SUMS` on tag
- [ ] Generate notes from the changelog

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Tag creates a release
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`

---

## Sign release artifacts with Sigstore

**Labels:** ci, security, phase-2, difficulty: medium

### Summary

Consumers want to verify artifacts.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: verification command works.

### Requirements

- [ ] Sign WASM and checksums with cosign keyless
- [ ] Document verification

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Verification command works
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Generate an SBOM for the SDK and contracts

**Labels:** ci, security, phase-2, good first issue

### Summary

Supply-chain transparency.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: sBOM attached.

### Requirements

- [ ] Produce CycloneDX output on release
- [ ] Attach to the release

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] SBOM attached
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add a CodeQL scan for TypeScript

**Labels:** ci, security, phase-2, good first issue

### Summary

Static analysis.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: scan runs weekly.

### Requirements

- [ ] Enable CodeQL for `sdk/`
- [ ] Triage findings

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Scan runs weekly
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add OpenSSF Scorecard and fix low scores

**Labels:** ci, security, phase-2, difficulty: medium

### Summary

Public security posture.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: score published in README.

### Requirements

- [ ] Add the workflow
- [ ] Fix top findings

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Score published in README
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add branch protection documentation and enforcement checklist

**Labels:** documentation, ci, phase-2, good first issue

### Summary

Rules should be written down.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: document merged.

### Requirements

- [ ] Document required checks and reviews
- [ ] Add a settings checklist

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Document merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- `.github/workflows/`

---

## Add a Docker image for local development network

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Local testing without testnet.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: `docker compose up` starts a local network.

### Requirements

- [ ] Build an image with stellar quickstart
- [ ] Add compose file
- [ ] Document

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] `docker compose up` starts a local network
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a local deploy script targeting the quickstart network

**Labels:** tooling, phase-2, good first issue

### Summary

Companion to the image.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: local deploy succeeds.

### Requirements

- [ ] Extend `deploy.sh` with a `local` network
- [ ] Document funding

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Local deploy succeeds
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a script to export a contract's current state for debugging

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Helpful for support.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: output is readable.

### Requirements

- [ ] Dump instance and key storage entries to JSON

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Output is readable
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a log decoder for diagnostic events

**Labels:** tooling, phase-2, good first issue

### Summary

Debug output is opaque.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: decoder prints readable data.

### Requirements

- [ ] Decode diagnostic events from a transaction hash

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Decoder prints readable data
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add Prometheus metrics to the reference indexer

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Operators need metrics.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: metrics endpoint works.

### Requirements

- [ ] Expose lag, events per second and errors

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Metrics endpoint works
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add a Grafana dashboard definition for protocol health

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Visibility.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: dashboard imports cleanly.

### Requirements

- [ ] Provide JSON for balances, mints and pauses

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Dashboard imports cleanly
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Add alert rules for admin actions

**Labels:** tooling, security, phase-2, difficulty: medium

### Summary

Detect misuse.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: test alert fires.

### Requirements

- [ ] Define alerts on rotations, pauses and overrides

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Test alert fires
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] Findings, assumptions and residual risks are written down in `docs/`
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- [Phase 1 internal review](docs/audits/2026-09-14-internal-review-phase1.md)
- [Security notes](docs/SECURITY.md)

---

## Add Discord or Slack webhook notifications for protocol events

**Labels:** tooling, phase-2, good first issue

### Summary

Team awareness.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: webhook receives a test event.

### Requirements

- [ ] Add a notifier consuming indexer events

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Webhook receives a test event
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Document RPC provider selection and failover

**Labels:** documentation, phase-2, good first issue

### Summary

RPC reliability.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Compare public and paid providers
- [ ] Describe failover in the SDK

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add an RPC failover option to the SDK

**Labels:** sdk, phase-2, difficulty: medium

### Summary

Single RPC is a point of failure.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: failover test passes.

### Requirements

- [ ] Accept a list of URLs
- [ ] Rotate on error
- [ ] Add tests

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Failover test passes
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)

---

## Document fee budgeting and sponsored transaction operations

**Labels:** documentation, phase-2, good first issue

### Summary

Sponsors need cost planning.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Explain fee bumps, budgets and monitoring

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a TTL extender bot for critical ledger entries

**Labels:** tooling, phase-2, difficulty: medium

### Summary

Entries archive when TTL lapses.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: bot extends a test entry.

### Requirements

- [ ] Scan manifest contracts
- [ ] Extend TTL before a threshold
- [ ] Log actions

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Bot extends a test entry
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)

---

## Document state archival and restore procedures

**Labels:** documentation, phase-2, difficulty: medium

### Summary

Soroban archives entries.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: doc merged.

### Requirements

- [ ] Explain detection and restore with CLI examples

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Doc merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Test archival and restore for a contract instance on testnet

**Labels:** contracts, testing, phase-2, difficulty: medium

### Summary

Prove the procedure.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: result documented.

### Requirements

- [ ] Let TTL lapse
- [ ] Restore
- [ ] Verify functions

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Result documented
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `cargo fmt --all`, `cargo clippy` and `cargo test --all --features testutils` pass
- [ ] New public functions have rustdoc, tests and an emitted event where state changes
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `contracts/`
- [ADR index](docs/architecture/)
- [Security notes](docs/SECURITY.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add a disaster recovery plan for lost infrastructure

**Labels:** documentation, phase-2, difficulty: medium

### Summary

Operational resilience.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: plan merged.

### Requirements

- [ ] Cover RPC loss, key loss and indexer rebuild

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Plan merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add a data retention and privacy policy for the indexer

**Labels:** documentation, phase-2, good first issue

### Summary

Compliance with privacy laws.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: policy merged.

### Requirements

- [ ] Describe stored data and deletion

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Policy merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Add load testing for the reference indexer

**Labels:** tooling, testing, phase-2, difficulty: medium

### Summary

Know the limits.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: report merged.

### Requirements

- [ ] Replay a large event set
- [ ] Report throughput

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The tool runs from a clean checkout and its usage is documented
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `scripts/`
- [Makefile](Makefile)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add a performance benchmark suite for the SDK

**Labels:** sdk, testing, phase-2, good first issue

### Summary

Track regressions.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: benchmarks run in CI.

### Requirements

- [ ] Benchmark parsing and building
- [ ] Store results

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Benchmarks run in CI
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] New tests fail when the behavior they guard is broken
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- `contracts/*/tests/`
- `sdk/tests/`

---

## Add an end to end release rehearsal on testnet

**Labels:** ci, documentation, phase-2, difficulty: medium

### Summary

De-risk the real release.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: rehearsal report merged.

### Requirements

- [ ] Run the full release flow with test keys
- [ ] Record problems

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Rehearsal report merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] The workflow runs green on a pull request and uses pinned action versions
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `.github/workflows/`
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Create a v1.0 SDK stabilization checklist

**Labels:** sdk, documentation, phase-2, good first issue

### Summary

Gate for the 1.0 release.

### Context

Before mainnet the project needs repeatable releases, nightly integration runs, observability for deployed contracts and clear operational documentation.

This issue belongs to **Stage 11: Operations, Observability and Release Engineering**.

### Objective

Deliver the change described above so that: checklist merged.

### Requirements

- [ ] List API freeze, docs, tests and migration notes

### Out of Scope

Feature work on contracts or SDK APIs.

### Acceptance Criteria

- [ ] Checklist merged
- [ ] CI is green and a maintainer has reviewed the change

### Definition of Done

- [ ] `npm run lint`, `npm run typecheck` and `npm test` pass in `sdk/`
- [ ] Public exports have TSDoc and the SDK README is updated if the API changed
- [ ] Markdown renders correctly and all links resolve
- [ ] The pull request links this issue and updates `CHANGELOG.md` when behavior changes

### References

- `sdk/src/`
- [SDK README](sdk/README.md)
- [docs/](docs/)
- [CONTRIBUTING.md](CONTRIBUTING.md)

---
