import { StrKey } from "@stellar/stellar-sdk";
import { createHash } from "node:crypto";

// ─── Address Utilities ────────────────────────────────────────────────────────

/** Returns true if the string is a valid Stellar public key (G-address). */
export function isValidStellarAddress(address: string): boolean {
  try {
    return StrKey.isValidEd25519PublicKey(address);
  } catch {
    return false;
  }
}

/** Returns true if the string is a valid Soroban contract ID (C-address). */
export function isValidContractId(id: string): boolean {
  try {
    return StrKey.isValidContract(id);
  } catch {
    return false;
  }
}

// ─── Amount Utilities ─────────────────────────────────────────────────────────

/** Largest `decimals` the contract accepts, mirroring `MAX_DECIMALS` in `rwa-asset`. */
const MAX_DECIMALS = 18;

/** Plain decimal notation: an optional sign, digits, and at most one fractional part. */
const DECIMAL_AMOUNT = /^-?\d+(\.\d+)?$/;

function assertDecimals(decimals: number): void {
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > MAX_DECIMALS) {
    throw new RangeError(`decimals must be an integer from 0 to ${MAX_DECIMALS}, got ${decimals}`);
  }
}

/**
 * Convert a human-readable decimal amount to base units at `decimals` places.
 * Defaults to 7 (Stellar's stroop convention). RWA assets may use a different
 * number of decimals — the contract allows up to 18 — so pass the asset's
 * decimals when it differs from 7.
 *
 * The result goes to `mint`, `transfer` or `approve`, where a silently wrong
 * amount moves real value, so this refuses input rather than guessing at it
 * (IR-06):
 *
 * - `amount` must be plain decimal notation. An exponent, hex, whitespace,
 *   thousands separators and a second decimal point are all rejected.
 * - Digits beyond `decimals` are rejected unless they are all zeros.
 * - A `number` must be a safe integer. Pass fractional or large amounts as a
 *   string: a `number` has already lost precision before this function sees it.
 */
export function toStroops(amount: number | string, decimals = 7): bigint {
  assertDecimals(decimals);
  if (typeof amount === "number" && !Number.isSafeInteger(amount)) {
    throw new RangeError(
      `${amount} is not a safe integer; pass fractional or large amounts as a string`,
    );
  }

  const text = String(amount);
  if (!DECIMAL_AMOUNT.test(text)) {
    throw new SyntaxError(
      `Invalid amount ${JSON.stringify(text)}: expected plain decimal notation such as "12.5"`,
    );
  }

  const [integer = "", fraction = ""] = text.split(".");
  if (/[^0]/.test(fraction.slice(decimals))) {
    throw new RangeError(
      `Amount ${text} has ${fraction.length} decimal places, more than the ${decimals} allowed`,
    );
  }
  return BigInt(integer + fraction.slice(0, decimals).padEnd(decimals, "0"));
}

/**
 * Convert base units back to a human-readable decimal string.
 * Defaults to 7 (Stellar's stroop convention); pass the asset's decimals when
 * it differs. Trailing fractional zeros are dropped.
 */
export function fromStroops(stroops: bigint, decimals = 7): string {
  assertDecimals(decimals);
  const sign = stroops < 0n ? "-" : "";
  const digits = (stroops < 0n ? -stroops : stroops).toString().padStart(decimals + 1, "0");
  const integer = digits.slice(0, digits.length - decimals);
  const fraction = digits.slice(digits.length - decimals).replace(/0+$/, "");
  return `${sign}${integer}${fraction ? `.${fraction}` : ""}`;
}

// ─── Document Hashing ─────────────────────────────────────────────────────────

/** SHA-256 hash of a Buffer or string, returned as hex. */
export function sha256Hex(data: Buffer | string): string {
  return createHash("sha256")
    .update(typeof data === "string" ? Buffer.from(data, "utf8") : data)
    .digest("hex");
}

/**
 * Convert a hex string to a 32-byte Uint8Array.
 *
 * Accepts exactly 64 hex digits. Parsing digit pairs used to turn anything
 * non-hex into zero bytes, so a mistyped legal-document hash was stored as a
 * different, valid-looking one (IR-06).
 */
export function hexToBytes32(hex: string): Uint8Array {
  if (!/^[0-9a-fA-F]{64}$/.test(hex)) {
    throw new Error("Expected 64-char hex string");
  }
  return Uint8Array.from(Buffer.from(hex, "hex"));
}

// ─── Formatting ───────────────────────────────────────────────────────────────

/** Truncate a Stellar address for display: G12345...67890 */
export function truncateAddress(address: string, chars = 5): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}
