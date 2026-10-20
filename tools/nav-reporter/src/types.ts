import type { OracleAsset } from "@stellarforge-protocol/sdk";

/**
 * Core appraisal payload signed by an institutional appraiser.
 */
export interface AppraisalData {
  /** The asset this appraisal prices. */
  asset: OracleAsset;
  /** Decimal string representation of the price (e.g. "123456.78"). */
  price: string;
  /** ISO-8601 UTC timestamp when the valuation was observed. */
  observedAt: string;
  /** Identifier of the certified appraisal firm. */
  appraiser: string;
  /** SHA-256 hash of the complete offline appraisal documentation. */
  documentSha256: string;
}

/**
 * Signed appraisal file structure containing payload and detached Ed25519 signature.
 */
export interface SignedAppraisalFile {
  payload: AppraisalData;
  /** Ed25519 detached signature as a hex or base64 string. */
  signature: string;
  /** Public key of the appraiser (Stellar G-address or hex). */
  publicKey: string;
}

/**
 * Standard exit codes returned by the CLI.
 */
export enum ExitCode {
  Success = 0,
  AuthFailure = 2,
  PreflightFailure = 3,
  SubmissionFailure = 4,
  UnknownOutcome = 5,
}

/**
 * Structured result object for CLI output.
 */
export interface ReportingResult {
  success: boolean;
  exitCode: ExitCode;
  action: "submitted" | "dry_run" | "idempotent_duplicate" | "rejected";
  txHash?: string;
  ledger?: number;
  tick?: string;
  error?: string;
  details?: Record<string, unknown>;
}
