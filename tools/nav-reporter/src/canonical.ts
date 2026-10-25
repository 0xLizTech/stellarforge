import { Keypair } from "@stellar/stellar-sdk";
import type { AppraisalData, SignedAppraisalFile } from "./types.js";

/**
 * Deterministically serializes a JSON value into RFC 8785 canonical JSON
 * by recursively sorting object keys alphabetically.
 */
export function canonicalJsonStringify(value: unknown): string {
  if (value === null || typeof value !== "object") {
    return JSON.stringify(value);
  }

  if (Array.isArray(value)) {
    return "[" + value.map((item) => canonicalJsonStringify(item)).join(",") + "]";
  }

  const entries = Object.entries(value as Record<string, unknown>).sort(
    ([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)
  );

  return (
    "{" +
    entries
      .map(([k, v]) => JSON.stringify(k) + ":" + canonicalJsonStringify(v))
      .join(",") +
    "}"
  );
}

/**
 * Verifies the Ed25519 signature on a signed appraisal against an expected appraiser key.
 */
export function verifyAppraisalSignature(
  signedFile: SignedAppraisalFile,
  expectedPublicKey?: string
): boolean {
  try {
    const pubKey = expectedPublicKey || signedFile.publicKey;
    if (expectedPublicKey && signedFile.publicKey !== expectedPublicKey) {
      return false;
    }

    const canonicalBytes = Buffer.from(canonicalJsonStringify(signedFile.payload), "utf8");
    const sigBuffer = Buffer.from(
      signedFile.signature,
      signedFile.signature.length === 128 ? "hex" : "base64"
    );

    const keypair = Keypair.fromPublicKey(pubKey);
    return keypair.verify(canonicalBytes, sigBuffer);
  } catch {
    return false;
  }
}
