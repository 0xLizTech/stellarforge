# Requests for Comments (RFCs)

StellarForge uses an RFC process for significant, protocol-level changes. An RFC
is a written proposal that lets maintainers and the community agree on *what* to
build and *why* before any code is written. Once an RFC is accepted, its decision
is recorded as an ADR in [`docs/architecture/`](../architecture/) when it is
architectural in nature.

## When an RFC is required

Write an RFC before proposing:

- Changes to a deployed contract's public interface (entry points, types, events).
- Changes to storage layout, TTL policy, or migrations between versions.
- New contracts, token standards, or cross-contract integration patterns.
- Changes to phase scope, roadmap commitments, or the governance process itself.

An RFC is **not** required for:

- Bug fixes that restore documented behavior.
- Documentation, CI, tooling, or test-only changes.
- Internal refactors that do not change any observable behavior or interface.
- Dependency bumps with no API impact.

When in doubt, open a discussion or draft RFC — it is cheaper than disagreeing
after implementation.

## Lifecycle

1. **Draft** — copy [`0000-template.md`](0000-template.md) to
   `docs/rfcs/NNNN-short-title.md`, where `NNNN` is the next free number (check
   the directory and any open RFC pull requests). Fill in every section; delete
   none of them. Set `Status: Draft`.
2. **Review** — open a pull request. Discussion happens on the PR and any linked
   issue. Update the document as consensus forms; keep the PR's history honest
   by pushing new commits rather than force-pushing over review context.
3. **Accepted / Rejected** — a maintainer sets the final status in the document
   and merges (accepted) or closes (rejected) the PR. A rejected RFC is kept in
   the repository for its historical record.
4. **Implemented / Superseded** — once implemented, the RFC is updated to point
   at the implementation. If a later RFC changes the decision, the earlier RFC
   is marked `Superseded by NNNN`.

## Status values

| Status | Meaning |
| --- | --- |
| `Draft` | Open for comment; not yet a commitment. |
| `Accepted` | Agreed; implementation may begin. |
| `Rejected` | Considered and declined; kept for the record. |
| `Implemented` | Accepted *and* shipped; links to the implementation. |
| `Superseded by NNNN` | A later RFC replaced this decision. |

## Relationship to ADRs

RFCs and ADRs are complementary:

- An **RFC** is a proposal — it can be edited during review, and it can be
  rejected.
- An **ADR** is an immutable decision record written after the decision, using
  [`docs/architecture/000-template.md`](../architecture/000-template.md).

Not every accepted RFC needs an ADR (for example, a standards-tracking RFC), and
small architectural decisions may go straight to an ADR without an RFC.

## Numbering

Numbers are assigned in the pull request. If two PRs collide on a number, the
later one renumbers before merge. Gaps are fine; numbers are never reused.
