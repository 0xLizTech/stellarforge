import { Keypair, StrKey } from "@stellar/stellar-sdk";
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

/** Convert a human-readable decimal amount to stroops (7 decimal places). */
export function toStroops(amount: number | string): bigint {
  const [integer, fraction = ""] = String(amount).split(".");
  const paddedFraction = fraction.padEnd(7, "0").slice(0, 7);
  return BigInt(integer + paddedFraction);
}

/** Convert stroops back to a human-readable decimal string. */
export function fromStroops(stroops: bigint): string {
  const str = stroops.toString().padStart(8, "0");
  const integer = str.slice(0, -7) || "0";
  const fraction = str.slice(-7);
  return `${integer}.${fraction}`.replace(/\.?0+$/, "");
}

// ─── Document Hashing ─────────────────────────────────────────────────────────

/** SHA-256 hash of a Buffer or string, returned as hex. */
export function sha256Hex(data: Buffer | string): string {
  return createHash("sha256")
    .update(typeof data === "string" ? Buffer.from(data, "utf8") : data)
    .digest("hex");
}

/** Convert a hex string to a 32-byte Uint8Array. */
export function hexToBytes32(hex: string): Uint8Array {
  if (hex.length !== 64) throw new Error("Expected 64-char hex string");
  const bytes = new Uint8Array(32);
  for (let i = 0; i < 32; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

// ─── Formatting ───────────────────────────────────────────────────────────────

/** Truncate a Stellar address for display: G12345...67890 */
export function truncateAddress(address: string, chars = 5): string {
  if (address.length <= chars * 2 + 3) return address;
  return `${address.slice(0, chars)}...${address.slice(-chars)}`;
}
