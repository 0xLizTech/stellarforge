# Resource Benchmarks

`resource-baseline.md` records the CPU, memory, and ledger read/write cost of
every public entry point in every Phase 1/2 contract (`rwa-asset`,
`compliance`, `registry`, `governance`, `oracle-adapter`), measured against a
real Soroban host.

## Why

Before this, a change to an entry point's cost had no tracked baseline, so a
regression (an accidental extra read, an unnecessary extend call, a loop that
grew) would pass unnoticed unless it happened to blow through a mainnet
resource limit outright. This gives reviewers something to diff a PR's
numbers against.

## How it is generated

```sh
cargo run -p stellarforge-benchmark > docs/benchmarks/resource-baseline.md
```

The tool (`tools/benchmark/`) builds a real `soroban-sdk` test `Env` for each
contract, performs the minimum setup each entry point needs (an existing
asset to read, an existing KYC record to revoke, and so on), invokes every
entry point once, and reads its cost via `env.cost_estimate().resources()`,
which reports the most recently completed top-level invocation. Mainnet
per-invocation resource limits (`NetworkInvocationResourceLimits::mainnet()`)
are enforced throughout the run, so a change that would fail those limits
outright fails the benchmark run itself, not just this report's numbers.

## What this is, and is not

One representative call per entry point, not an exhaustive cost model. It
does not capture every branch an entry point can take (an empty vs. a
populated `list_assets` page, a cold vs. a warm KYC record lookup, and so on
cost differently and are not separately recorded here). Its job is catching
a meaningful drift in an entry point's cost from what it used to be, not
predicting the exact fee a specific mainnet call will pay — `cost_estimate()`
itself does not model VM instantiation, Wasm-specific costs, or transaction
size, and this report inherits that same approximation.

## Updating after a contract change

Regenerate with the command above and commit the result alongside the
contract change that caused it to move. A diff that only touches entries for
contracts/entry points outside the PR's own change is worth a second look
before committing; it usually means the benchmark run picked up unrelated
local state rather than the PR's own effect.
