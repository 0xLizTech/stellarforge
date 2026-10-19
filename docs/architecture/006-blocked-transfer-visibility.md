# ADR-006: Blocked Transfer Visibility

**Status:** Accepted
**Date:** 2026-10-16
**Authors:** StellarForge Core Team

---

## Context

`rwa-asset`'s `require_compliant` (ADR-004) rejects a transfer by calling `panic_with_error!`. A panic aborts the whole host transaction: nothing it wrote is kept, and nothing it published is kept either, because an event is itself part of the transaction's effects and effects do not survive a revert. An indexer, a compliance dashboard, or an operator watching for "who is this policy actually blocking" sees nothing — not a `Blocked` event, not a failed-transaction placeholder with useful detail, nothing. The only visibility into a rejected transfer today is the submitter's own client-side error, which never reaches chain state or any event stream.

This issue asks for a diagnostic event emitted "only in simulation" — the idea being that a wallet or indexer simulating a transfer before submitting it could surface *why* it would fail, without that event needing to survive an on-chain revert, since simulation output never commits to a ledger in the first place.

## Decision

**No code change.** The simulation-only diagnostic event is not implementable, and nothing is added in its place.

The reason is structural, not a missing SDK feature we could ask for: Soroban's host gives a contract no way to distinguish "I am being simulated for a `simulateTransaction` RPC call" from "I am executing as part of a submitted transaction." The same WASM module runs identically either way, against the same host interface, with no flag, context field, or ledger-state difference a contract can read. This is deliberate on Soroban's part — simulation is supposed to predict the real execution path exactly, including every panic, which it could not do if a contract were allowed to special-case its own simulation. There is consequently no `if simulating { event } else { nothing }` branch to write.

What already exists covers the practical need, short of an on-chain event:

- **`screen`/`is_compliant`** (ADR-004) are both public, authless queries. Any wallet or indexer can call `is_compliant(party, min_level)` directly, before building a transfer, and get the exact true/false `require_compliant` will act on, with no simulation involved at all. This answers "would this transfer be blocked, and by which party" more directly than a diagnostic event could, since it names the specific address and level rather than a generic "blocked" signal.
- **A standard `simulateTransaction` RPC call** already returns the panic's error code (`RwaError::NotCompliant`) in its response, which is enough for a wallet to show "this would fail: compliance" before the holder signs anything. That failure detail exists today independent of this ADR; it was never actually missing, only not visible in an *event stream*, which simulation output structurally cannot feed, since it commits nothing.

## Alternatives Considered

**Emit the event before the compliance check, unconditionally, and panic afterward.** Rejected. A panic rolls back everything in the transaction, including every event published earlier in it (Soroban does not have "events survive a revert" semantics — commit is all-or-nothing). This would not actually get the event onto the ledger for a rejected transfer; it would just add dead code that never has an observable effect.

**Return a result/error code from `transfer` instead of panicking, carrying a reason.** Rejected for this issue's scope. It would make blocked-transfer visibility real (a caller gets a value back, which it can act on or forward), but it is a breaking change to `rwa-asset`'s transfer interface, affects every caller of `transfer`/`transfer_from`, and trades the fail-loudly guarantee the rest of ADR-004's design relies on for a pattern none of this protocol's other entry points use. Worth its own ADR and issue if the protocol wants it; out of scope for a diagnostic-visibility follow-up.

**Have the compliance contract itself log an off-chain-visible marker before `screen` returns `false`.** Rejected. `screen`'s return value already is that marker — `require_compliant` reads it and decides to panic. Adding a second signal (an event from inside `screen`) for the same boolean the caller already receives duplicates information without adding any new visibility, and ties `screen`'s one-function surface (ADR-004) to an event schema only `rwa-asset` would ever consume.

**Add an off-chain indexer convention: watch for failed `InvokeHostFunction` operations and decode the error code from the result meta.** Not rejected, but explicitly out of scope: this is infrastructure a dashboard operator builds against Horizon/RPC, not a change to any contract in this repository. Noted here as the closest thing to "diagnostic visibility" that does exist, for whoever picks this up operationally.

## Consequences

- **No contract code changes as a result of this ADR.** `rwa-asset`'s `require_compliant` is unchanged; it still panics via `RwaError::NotCompliant`, with the behavior and reasoning ADR-004 already documents.
- **The practical answer to "why would my transfer be blocked" is: call `is_compliant` on both parties first.** This is already possible today and requires no new entry point; it is simply not automatic the way an event would have been. A wallet integration that wants to warn a user before they attempt a doomed transfer should call it proactively.
- **A future protocol version that wants richer failure detail should change `transfer`'s return type, not look for a simulation-only event again.** The technical reason this issue's original approach cannot work is permanent (it is how the host is built, not a version-specific gap), so any future revisit of "blocked transfers are invisible" should start from the result-code alternative above, not from simulation detection.
