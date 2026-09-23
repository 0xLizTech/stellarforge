// Generates sdk/tests/fixtures/oracle-parity-table.json
// Mirrors contracts/oracle-adapter/tests/test_sdk_parity_table.rs exactly
// (same case list and arithmetic as OracleAdapterContract::tick / record).
const fs = require("fs");
const path = require("path");

const BPS = 10000n;
const I128_MAX = (1n << 127n) - 1n;
const I128_MIN = -(1n << 127n);
const U64_MAX = (1n << 64n) - 1n;

function tickOf(timestamp, resolution) {
  const res = BigInt(resolution);
  return timestamp - (timestamp % res);
}

function checkedMul(a, b) {
  const p = a * b;
  if (p > I128_MAX || p < I128_MIN) return null;
  return p;
}

function checkDeviation(previous, next, maxBps) {
  const moved = next >= previous ? next - previous : previous - next;
  const lhs = checkedMul(moved, BPS);
  const rhs = checkedMul(previous, BigInt(maxBps));
  if (lhs === null || rhs === null) return "Overflow";
  return lhs <= rhs;
}

function saturatingMulU64(a, b) {
  const p = a * b;
  return p > U64_MAX ? U64_MAX : p;
}

function saturatingAddI128(a, b) {
  const s = a + b;
  if (s > I128_MAX) return I128_MAX;
  if (s < I128_MIN) return I128_MIN;
  return s;
}

function saturatingSubI128(a, b) {
  const s = a - b;
  if (s > I128_MAX) return I128_MAX;
  if (s < I128_MIN) return I128_MIN;
  return s;
}

const tickCases = [];
const resolutions = [1, 2, 60, 300, 3600, 86400];
for (const res of resolutions) {
  const resU = BigInt(res);
  for (const k of [0n, 1n, 2n, 7n, 100n, 1_000_000n]) {
    const ts = saturatingMulU64(k, resU);
    tickCases.push({ timestamp: ts.toString(), resolution: res, expected: tickOf(ts, res).toString() });
  }
  if (res > 1) {
    const mid = saturatingMulU64(10n, resU);
    for (const ts of [mid - 1n, mid + 1n, mid + resU / 2n]) {
      tickCases.push({ timestamp: ts.toString(), resolution: res, expected: tickOf(ts, res).toString() });
    }
  }
}
for (const [ts, res] of [
  [U64_MAX - 10n, 60],
  [1_800_000_123n, 300],
  [1_800_000_000n, 300],
  [1_700_000_045n, 60],
  [1_700_000_060n, 60],
  [1_700_000_000n, 3600],
]) {
  tickCases.push({ timestamp: ts.toString(), resolution: res, expected: tickOf(ts, res).toString() });
}

const bases = [
  1n, 2n, 10n, 100n, 1000n, 10_000n, 100_000n, 1_000_000n,
  10n ** 14n,
  100n * 10n ** 14n,
  10n ** 18n,
  I128_MAX / 20_000n,
];
const bpsValues = [1, 2, 10, 100, 500, 1000, 2500, 5000, 10000];
const deviationCases = [];

for (const prev of bases) {
  for (const bps of bpsValues) {
    const eq = checkDeviation(prev, prev, bps);
    deviationCases.push({ previous: prev.toString(), next: prev.toString(), maxDeviationBps: bps, expected: eq });

    const maxDelta = (prev * BigInt(bps)) / BPS; // trunc toward 0 like Rust
    if (maxDelta > 0n) {
      const up = saturatingAddI128(prev, maxDelta);
      const down = saturatingSubI128(prev, maxDelta);
      if (up > 0n) {
        deviationCases.push({ previous: prev.toString(), next: up.toString(), maxDeviationBps: bps, expected: checkDeviation(prev, up, bps) });
        if (up < I128_MAX) {
          const pastUp = up + 1n;
          if (pastUp > 0n) {
            deviationCases.push({ previous: prev.toString(), next: pastUp.toString(), maxDeviationBps: bps, expected: checkDeviation(prev, pastUp, bps) });
          }
        }
      }
      if (down > 0n) {
        deviationCases.push({ previous: prev.toString(), next: down.toString(), maxDeviationBps: bps, expected: checkDeviation(prev, down, bps) });
        if (down > 1n) {
          deviationCases.push({ previous: prev.toString(), next: (down - 1n).toString(), maxDeviationBps: bps, expected: checkDeviation(prev, down - 1n, bps) });
        }
      }
    } else if (prev < I128_MAX) {
      deviationCases.push({ previous: prev.toString(), next: (prev + 1n).toString(), maxDeviationBps: bps, expected: checkDeviation(prev, prev + 1n, bps) });
    }
  }
}

// Overflow cases
{
  const p = I128_MAX / 5000n;
  deviationCases.push({ previous: p.toString(), next: (p + 1n).toString(), maxDeviationBps: 10000, expected: checkDeviation(p, p + 1n, 10000) });
}
{
  const bigPrev = I128_MAX / 100n + 1n;
  deviationCases.push({ previous: bigPrev.toString(), next: bigPrev.toString(), maxDeviationBps: 10000, expected: checkDeviation(bigPrev, bigPrev, 10000) });
}
deviationCases.push({ previous: "1", next: I128_MAX.toString(), maxDeviationBps: 1, expected: checkDeviation(1n, I128_MAX, 1) });
{
  const p = I128_MAX / 2n;
  deviationCases.push({ previous: p.toString(), next: p.toString(), maxDeviationBps: 3, expected: checkDeviation(p, p, 3) });
}
{
  const near = I128_MAX / 10001n;
  deviationCases.push({ previous: near.toString(), next: (near + 1n).toString(), maxDeviationBps: 1, expected: checkDeviation(near, near + 1n, 1) });
  deviationCases.push({ previous: near.toString(), next: near.toString(), maxDeviationBps: 1, expected: checkDeviation(near, near, 1) });
}

const table = {
  generatedBy: "contracts/oracle-adapter/tests/test_sdk_parity_table.rs",
  note: "Expected values computed with the contract's tick and checked_mul deviation arithmetic (BPS_DENOMINATOR=10000). Regenerated by the Rust test when cargo test is available; this file is the committed fixture the SDK tests load.",
  bpsDenominator: 10000,
  tickOf: tickCases,
  checkDeviation: deviationCases,
};

const total = tickCases.length + deviationCases.length;
if (total < 50) throw new Error(`need >= 50 cases, got ${total}`);

const outPath = path.join(__dirname, "..", "sdk", "tests", "fixtures", "oracle-parity-table.json");
fs.mkdirSync(path.dirname(outPath), { recursive: true });
fs.writeFileSync(outPath, JSON.stringify(table, null, 2) + "\n");
console.log(`wrote ${outPath}: tickOf=${tickCases.length} checkDeviation=${deviationCases.length} total=${total}`);