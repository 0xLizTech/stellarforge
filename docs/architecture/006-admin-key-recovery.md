# ADR-006: Admin Key Recovery

**Status:** Accepted
**Date:** 2026-10-16
**Authors:** StellarForge Core Team

---

## Context

ADR-002 fixed how admin transfer is authorized — both the outgoing and incoming
admin sign, in one transaction — and closed the *wrong-address* hazard: a
handover nobody controls cannot complete, because the incoming side has to
prove it holds the key by signing for it.

It does not address a different failure: the admin key existing, being
correct, and being **lost** — a hardware wallet destroyed, a seed phrase never
backed up, an operator who leaves without handing over credentials. `admin()`
still returns a real address; nothing is wrong with contract state. There is
simply no longer anyone who can produce that address's signature. Every
admin-gated entry point across the five Phase 1/2 contracts — `transfer_admin`
itself included — requires `require_auth()` on the current admin, so losing
that key locks every one of them permanently. There is no path back in from
outside the contract, because the contract has no concept of an outside.

This is IR-04's residual risk, named but deliberately not solved there: the
review fixed admin *rotation* (so a live admin is not stuck with one key
forever) but did not give a *dead* admin a way back.

### Why this is being revisited now, and not folded into #125

ADR-002 separately rejected a two-step `propose_admin`/`accept_admin` handover
for the wrong-address case, on grounds that no longer hold: it required two
signatures in one Soroban transaction, which the SDK could not build
(tracked as #31). #31 is now closed — `sdk/src/sponsored.ts` and the
`buildSponsored*Tx` family ship exactly the authorization-entry mechanism
ADR-002's *Consequences* section named as "the strongest argument for
revisiting the two-step alternative." That is #125's concern: whether
`transfer_admin` should become a two-step handover now that the SDK can
reach it at all (today it is reachable only from `stellar-cli`).

Recovering from a **lost** key is a different problem with a different
answer, and conflating the two would make neither decision clean. A two-step
handover still requires the admin's own signature to start it; it has
nothing to offer once that signature no longer exists. This ADR is scoped to
that gap alone.

## Decision

### A recovery guardian, activated only after a timelock, with no change to
### the single admin model

Each contract gains a second, lower-privilege address — the **guardian** —
set once at deploy time alongside `admin` and rotatable only by the admin
(the same dual-auth rotation ADR-002 already specifies, reused rather than
reinvented). The guardian has exactly one power: `initiate_recovery(new_admin)`,
which starts a timelock. Once the timelock elapses, anyone may call
`finalize_recovery()`, which installs `new_admin` as admin — no further
signature from the lost key is needed or possible, because the entire point
is that it cannot be produced.

The admin can cancel a pending recovery at any time with a single signature
(`cancel_recovery`), which is the guarantee that makes this safe to adopt: a
live admin that sees an unwanted recovery underway — whether the guardian
key was compromised or simply mistaken — can stop it unilaterally, and the
timelock is exactly the window that makes noticing possible before it takes
effect.

### Why a guardian, not the admin itself, initiates recovery

If recovering from a lost admin key required the admin's own signature, it
would not be recovery. The guardian exists specifically to be a *second*
key, held by a different party (a co-founder, a multisig of the other
protocol admins, a dedicated recovery service), whose only job is to still
be reachable when the first key is not. This is the same reasoning ADR-002
gives for why `issuer` is a separate role from `admin` in `rwa-asset`:
narrow, single-purpose delegation rather than a second copy of full
privilege.

### Why a timelock, and why it runs even though the guardian already signed

A guardian key can itself be compromised, coerced, or simply wrong about
whether the admin key is really lost. Requiring its signature proves intent
exists; it does not prove the intent is correct. The timelock is the
control for that: it converts "the guardian said so" into "the guardian
said so, and nobody with standing to object did, for a window long enough
to notice." `cancel_recovery` is what gives that window teeth — a timelock
nobody can interrupt is just a slower unconditional transfer.

### Default timelock: 7 days

Long enough that an admin who is merely offline — travel, an infrastructure
incident, a long weekend — has a realistic chance to return and notice. Short
enough that a protocol with a genuinely lost key is not paralyzed for a
season. Configurable per deployment via the same constructor-parameter
pattern `max_deviation_bps` already uses in `oracle-adapter`; this ADR fixes
the default, not a hardcoded constant.

### This is in addition to, not instead of, #125's two-step handover

A live admin uses `propose_admin`/`accept_admin` (once #125 lands) for a
routine, uncontested handover. The guardian/timelock path exists only for
the case that process cannot reach: an admin who cannot sign at all.
Implementing only one of the two leaves a real gap — the two-step flow alone
has nothing for a dead key, and a recovery path alone has nothing for the
common, non-emergency case of a live admin choosing to hand over cleanly.

## Alternatives Considered

**A pure timelock with no guardian — anyone can call `initiate_recovery`,
gated only by time and a fixed recipient chosen at deploy.** Rejected. A
fixed recovery recipient chosen once at deploy cannot adapt if *that*
address's key is also eventually lost or needs rotating, and "anyone can
initiate" means the timelock is the only defense, with no second signature
required to even start the clock. A compromised or simply malicious actor
could force every admin action through the delay window on a whim. The
guardian's signature is cheap insurance against a recovery process that
triggers on nothing.

**An M-of-N multisig as the admin itself, replacing the single-admin model
everywhere.** Not rejected on merit — it is a stronger primitive than a
guardian, since no single key loss is fatal in the first place. Rejected for
*this* ADR's scope because it is a materially larger change: every one of
the five contracts' `require_admin` call sites assumes a single `Address`,
and multisig threshold/signer-set state is a different shape of storage
with its own rotation and threshold-change semantics (see the drip-pool
precedent in a sibling protocol, which this team is aware of and did not
copy wholesale here because Phase 1/2's admin surface is narrower). A
guardian/timelock retrofits onto the existing single-admin storage with one
new address and one new timelock entry per contract; multisig does not.
Worth revisiting for Phase 3 if operational experience shows a single
guardian is itself too concentrated a risk — the two are not mutually
exclusive, since a multisig address can itself be set as the guardian (or
even as the admin) without any contract change.

**A social-recovery scheme (K-of-N guardians, not one).** Rejected for
Phase 1/2 on the same gap-vs-complexity basis as multisig above: it is a
strictly better guarantee (no single guardian key is a single point of
failure either) bought with threshold-storage complexity this ADR's
single-guardian design avoids entirely. A single guardian can itself *be* a
multisig-controlled address, which gets most of the benefit — K-of-N
guardian consensus — without the contract needing to know anything about
thresholds at all. Native K-of-N support is worth it only if that
composition proves insufficient in practice.

**No recovery mechanism; treat a lost admin key as an accepted residual
risk, same as ADR-003 treats a permanently non-upgradeable `rwa-asset`.**
Rejected. ADR-003's position is defensible specifically because the
alternative (an upgrade entry point) is a strictly larger attack surface
than the risk it defends against — an always-available upgrade hook is more
dangerous than an immutable contract with no holders-affecting bug. A
guardian/timelock recovery path is not symmetric with that tradeoff: its
only power is replacing one address under a delay anyone can watch and the
current admin can cancel, which is a far smaller surface than an upgrade
hook, for a failure mode (operator turnover, lost hardware) that is
mundane rather than hypothetical.

## Consequences

- **Every contract's deploy parameters grow by one address.** `__constructor`
  takes `guardian: Address` alongside `admin`, which is a breaking change to
  the constructor signature of every Phase 1/2 contract. Already-deployed
  instances are unaffected (ADR-003: no upgrade path), so this applies only
  to contracts deployed from this point forward.

- **A new timelock entry per contract**, holding at most one pending
  recovery at a time. A second `initiate_recovery` while one is already
  pending replaces it rather than queuing a second, mirroring how
  `propose_admin` (once #125 lands) is expected to behave for a pending
  transfer.

- **The guardian is a new key an operator must generate, store, and rotate
  with the same care as the admin key**, which is a real operational cost,
  not a free safety net. A guardian key that is lost alongside the admin key
  recovers nothing; a guardian key that is itself compromised can force a
  timelock-delayed takeover, mitigated but not eliminated by the admin's
  ability to cancel while still live.

- **`finalize_recovery` is permissionless by design** (anyone may call it
  once the timelock elapses), which keeps recovery from depending on the
  guardian remaining reachable for a second transaction. This matches
  `finalize`'s existing permissionless pattern in `governance`.

- **Implementation is tracked in follow-up issues, not this ADR.** Per
  #124's requirements, this document records the decision; #125 (two-step
  handover) and the recovery-guardian implementation are separate, linked
  issues so each can be reviewed, tested, and merged independently of this
  write-up.
