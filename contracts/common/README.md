# common

Shared building blocks for every StellarForge contract. A compile-time
library, never deployed on its own — the protocol's composability guarantee
is that each contract stays independently deployable, so only policy that
must be byte-for-byte identical across contracts belongs here.

## What's here

### `storage` — TTL policy

Soroban archives a ledger entry that is not periodically extended. The
`extend_instance`/`extend_persistent` helpers implement one shared policy so
every contract's write paths extend what they write the same way, rather
than each contract copying the constants and risking a drift like SF-2026-002
(a missing extension found and fixed once, here, instead of once per crate).

| Item | Kind | Description |
|---|---|---|
| `extend_instance(env)` | function | Extends the contract instance and its code to `INSTANCE_BUMP_AMOUNT` |
| `extend_persistent(env, key)` | function | Extends one persistent entry to the network's max TTL, if it exists; a no-op on a key never written |
| `persistent_bump_amount(env)` | function | The network's max TTL, read live rather than hardcoded |
| `persistent_lifetime_threshold(env)` | function | The remaining-TTL floor below which a persistent extension is issued |
| `DAY_IN_LEDGERS` | constant | Ledgers closed per day at the ~5 second target close time |
| `INSTANCE_BUMP_AMOUNT` / `INSTANCE_LIFETIME_THRESHOLD` | constants | How far ahead instance storage is extended, and the threshold that triggers it |
| `PERSISTENT_REFRESH_INTERVAL` | constant | How far a persistent entry may decay before a write path refreshes it |

Write paths call these and extend to the network maximum, since a key
written once and only read afterward has no other chance to be refreshed.
Read paths deliberately do not extend, so a query stays free of side
effects — see the module doc comment in
[`src/storage.rs`](src/storage.rs) for the full reasoning.

This crate exposes no errors or events of its own: it has no entry points,
so nothing here can fail or publish independently of the contract that
calls into it.
