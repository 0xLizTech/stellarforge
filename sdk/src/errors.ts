/**
 * Write-path failures and typed contract errors.
 *
 * What matters to automation is whether a retry can execute the operation a
 * second time. A plain `Error` (or {@link ContractError}) from the write path
 * means the outcome is known: the transaction was never accepted, or it was
 * applied and failed. {@link TransactionExpiredError} and
 * {@link TransactionOutcomeUnknownError} cover a transaction the network
 * accepted but that was not seen to settle.
 */

/**
 * The transaction's validity window closed without it being included in a
 * ledger.
 *
 * The RPC had already seen a ledger close after the transaction's `maxTime`
 * and still did not know the hash. No later ledger can include it, so
 * rebuilding and resubmitting cannot double-execute.
 */
export class TransactionExpiredError extends Error {
  readonly hash: string;

  /** Creates the expired-transaction error for a submitted transaction hash. */
  constructor(hash: string) {
    super(
      `Transaction ${hash} expired without being included in a ledger. ` +
        "It can no longer land, so it is safe to rebuild and resubmit.",
    );
    this.name = "TransactionExpiredError";
    this.hash = hash;
  }
}

/**
 * The transaction was accepted but had not settled when polling ended, and it
 * may still be included.
 *
 * This happens when the RPC has not yet seen a ledger past the transaction's
 * `maxTime`, for instance because it is lagging. Look the hash up before
 * retrying: a blind retry can execute a mint or transfer twice.
 */
export class TransactionOutcomeUnknownError extends Error {
  readonly hash: string;

  /** Creates the unresolved-outcome error for a submitted transaction hash. */
  constructor(hash: string) {
    super(
      `Transaction ${hash} was accepted but had not settled when polling ended, ` +
        "and it may still be included. Check this hash before retrying, or a retry " +
        "may execute the operation twice.",
    );
    this.name = "TransactionOutcomeUnknownError";
    this.hash = hash;
  }
}

/**
 * Thrown when an invalid Stellar address (G, C, or M) is passed to a client method.
 */
export class InvalidAddressError extends Error {
  readonly address: string;
  readonly parameterName?: string | undefined;

  constructor(address: string, parameterName?: string) {
    super(
      `Invalid Stellar address "${address}"${parameterName ? ` for parameter "${parameterName}"` : ""}. Expected a valid G, C, or M address.`,
    );
    this.name = "InvalidAddressError";
    this.address = address;
    if (parameterName !== undefined) this.parameterName = parameterName;
  }
}

// ─── Typed Contract Errors (#159) ─────────────────────────────────────────────

/**
 * Base class for typed Soroban contract errors.
 *
 * Preserves the raw numeric discriminant returned by the contract (`code`)
 * alongside its human-readable variant name (`codeName`), the contract
 * identifier (`contract`), and the original RPC error message (`rawError`).
 */
export class ContractError extends Error {
  /** Logical contract name (e.g. `"rwa-asset"`, `"compliance"`). */
  readonly contract: string;
  /** Numeric `u32` error discriminant returned by the Soroban contract. */
  readonly code: number;
  /** Human-readable enum variant name, or `"Unknown"` if unmapped. */
  readonly codeName: string;
  /** Original error string returned by the RPC or host, when available. */
  readonly rawError?: string;

  constructor(
    className: string,
    contract: string,
    code: number,
    codeName: string,
    rawError?: string,
  ) {
    const detail = rawError ? `: ${rawError}` : "";
    super(`${className} [${codeName} (#${code})]${detail}`);
    this.name = className;
    this.contract = contract;
    this.code = code;
    this.codeName = codeName;
    if (rawError !== undefined) {
      this.rawError = rawError;
    }
  }
}

// ── RWA Asset ─────────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `RwaError` in
 * `contracts/rwa-asset/src/error.rs`.
 */
export enum RwaAssetErrorCode {
  AlreadyInitialized = 1,
  NotInitialized = 2,
  NotIssuer = 3,
  InvalidAmount = 4,
  InsufficientBalance = 5,
  InsufficientAllowance = 6,
  ExceedsMaxSupply = 7,
  ContractPaused = 8,
  Overflow = 9,
  InvalidMetadata = 10,
  NotCompliant = 11,
  DecimalsImmutable = 12,
  InvalidSupplyCap = 13,
  InvalidExpiration = 14,
  InvalidComplianceLevel = 15,
  InvalidLegalDocHash = 16,
  GovernanceNotConfigured = 17,
  NoPendingCapIncrease = 18,
  NoPendingAdminTransfer = 19,
  NotForcedTransferRole = 20,
  InvalidReasonHash = 21,
  InvalidForcedTransferParty = 22,
  AddressFrozen = 23,
  AddressLocked = 24,
  ExceedsHolderCap = 25,
}

const RWA_ASSET_ERROR_NAMES: Readonly<Record<number, string>> = {
  [RwaAssetErrorCode.AlreadyInitialized]: "AlreadyInitialized",
  [RwaAssetErrorCode.NotInitialized]: "NotInitialized",
  [RwaAssetErrorCode.NotIssuer]: "NotIssuer",
  [RwaAssetErrorCode.InvalidAmount]: "InvalidAmount",
  [RwaAssetErrorCode.InsufficientBalance]: "InsufficientBalance",
  [RwaAssetErrorCode.InsufficientAllowance]: "InsufficientAllowance",
  [RwaAssetErrorCode.ExceedsMaxSupply]: "ExceedsMaxSupply",
  [RwaAssetErrorCode.ContractPaused]: "ContractPaused",
  [RwaAssetErrorCode.Overflow]: "Overflow",
  [RwaAssetErrorCode.InvalidMetadata]: "InvalidMetadata",
  [RwaAssetErrorCode.NotCompliant]: "NotCompliant",
  [RwaAssetErrorCode.DecimalsImmutable]: "DecimalsImmutable",
  [RwaAssetErrorCode.InvalidSupplyCap]: "InvalidSupplyCap",
  [RwaAssetErrorCode.InvalidExpiration]: "InvalidExpiration",
  [RwaAssetErrorCode.InvalidComplianceLevel]: "InvalidComplianceLevel",
  [RwaAssetErrorCode.InvalidLegalDocHash]: "InvalidLegalDocHash",
  [RwaAssetErrorCode.GovernanceNotConfigured]: "GovernanceNotConfigured",
  [RwaAssetErrorCode.NoPendingCapIncrease]: "NoPendingCapIncrease",
  [RwaAssetErrorCode.NoPendingAdminTransfer]: "NoPendingAdminTransfer",
  [RwaAssetErrorCode.NotForcedTransferRole]: "NotForcedTransferRole",
  [RwaAssetErrorCode.InvalidReasonHash]: "InvalidReasonHash",
  [RwaAssetErrorCode.InvalidForcedTransferParty]: "InvalidForcedTransferParty",
  [RwaAssetErrorCode.AddressFrozen]: "AddressFrozen",
  [RwaAssetErrorCode.AddressLocked]: "AddressLocked",
  [RwaAssetErrorCode.ExceedsHolderCap]: "ExceedsHolderCap",
};

/**
 * Typed error raised when an `rwa-asset` contract call fails with a contract
 * error code.
 */
export class RwaAssetError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "RwaAssetError",
      "rwa-asset",
      code,
      RWA_ASSET_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Compliance ────────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `ComplianceError` in
 * `contracts/compliance/src/error.rs`.
 */
export enum ComplianceErrorCode {
  AlreadyInitialized = 1,
  NotInitialized = 2,
  InvalidLevel = 3,
  AlreadyExpired = 4,
  InvalidJurisdiction = 5,
  ContractPaused = 6,
  BatchTooLarge = 7,
  NoPendingAdminTransfer = 8,
}

const COMPLIANCE_ERROR_NAMES: Readonly<Record<number, string>> = {
  [ComplianceErrorCode.AlreadyInitialized]: "AlreadyInitialized",
  [ComplianceErrorCode.NotInitialized]: "NotInitialized",
  [ComplianceErrorCode.InvalidLevel]: "InvalidLevel",
  [ComplianceErrorCode.AlreadyExpired]: "AlreadyExpired",
  [ComplianceErrorCode.InvalidJurisdiction]: "InvalidJurisdiction",
  [ComplianceErrorCode.ContractPaused]: "ContractPaused",
  [ComplianceErrorCode.BatchTooLarge]: "BatchTooLarge",
  [ComplianceErrorCode.NoPendingAdminTransfer]: "NoPendingAdminTransfer",
};

/**
 * Typed error raised when a `compliance` contract call fails with a contract
 * error code.
 */
export class ComplianceError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "ComplianceError",
      "compliance",
      code,
      COMPLIANCE_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Registry ──────────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `RegistryError` in
 * `contracts/registry/src/error.rs`.
 */
export enum RegistryErrorCode {
  AlreadyInitialized = 1,
  NotInitialized = 2,
  AssetNotFound = 3,
  PageTooLarge = 4,
  Overflow = 5,
  ContractPaused = 6,
  NoPendingAdminTransfer = 7,
}

const REGISTRY_ERROR_NAMES: Readonly<Record<number, string>> = {
  [RegistryErrorCode.AlreadyInitialized]: "AlreadyInitialized",
  [RegistryErrorCode.NotInitialized]: "NotInitialized",
  [RegistryErrorCode.AssetNotFound]: "AssetNotFound",
  [RegistryErrorCode.PageTooLarge]: "PageTooLarge",
  [RegistryErrorCode.Overflow]: "Overflow",
  [RegistryErrorCode.ContractPaused]: "ContractPaused",
  [RegistryErrorCode.NoPendingAdminTransfer]: "NoPendingAdminTransfer",
};

/**
 * Typed error raised when a `registry` contract call fails with a contract
 * error code.
 */
export class RegistryError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "RegistryError",
      "registry",
      code,
      REGISTRY_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Governance ────────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `GovernanceError` in
 * `contracts/governance/src/error.rs`.
 */
export enum GovernanceErrorCode {
  AlreadyInitialized = 1,
  NotInitialized = 2,
  ProposalNotFound = 3,
  AlreadyVoted = 4,
  ProposalNotActive = 5,
  VotingClosed = 6,
  VotingNotClosed = 7,
  InvalidWeight = 8,
  Overflow = 9,
  InvalidVotingPeriod = 10,
  TitleTooLong = 11,
  NotAuthorizedToCancel = 12,
  ProposalAlreadyHasVotes = 13,
  PageTooLarge = 14,
  InvalidDepositAmount = 15,
  NoPendingAdminTransfer = 16,
  InvalidQuorum = 17,
  InvalidPassThresholdBps = 18,
  DuplicateActiveProposal = 19,
}

const GOVERNANCE_ERROR_NAMES: Readonly<Record<number, string>> = {
  [GovernanceErrorCode.AlreadyInitialized]: "AlreadyInitialized",
  [GovernanceErrorCode.NotInitialized]: "NotInitialized",
  [GovernanceErrorCode.ProposalNotFound]: "ProposalNotFound",
  [GovernanceErrorCode.AlreadyVoted]: "AlreadyVoted",
  [GovernanceErrorCode.ProposalNotActive]: "ProposalNotActive",
  [GovernanceErrorCode.VotingClosed]: "VotingClosed",
  [GovernanceErrorCode.VotingNotClosed]: "VotingNotClosed",
  [GovernanceErrorCode.InvalidWeight]: "InvalidWeight",
  [GovernanceErrorCode.Overflow]: "Overflow",
  [GovernanceErrorCode.InvalidVotingPeriod]: "InvalidVotingPeriod",
  [GovernanceErrorCode.TitleTooLong]: "TitleTooLong",
  [GovernanceErrorCode.NotAuthorizedToCancel]: "NotAuthorizedToCancel",
  [GovernanceErrorCode.ProposalAlreadyHasVotes]: "ProposalAlreadyHasVotes",
  [GovernanceErrorCode.PageTooLarge]: "PageTooLarge",
  [GovernanceErrorCode.InvalidDepositAmount]: "InvalidDepositAmount",
  [GovernanceErrorCode.NoPendingAdminTransfer]: "NoPendingAdminTransfer",
  [GovernanceErrorCode.InvalidQuorum]: "InvalidQuorum",
  [GovernanceErrorCode.InvalidPassThresholdBps]: "InvalidPassThresholdBps",
  [GovernanceErrorCode.DuplicateActiveProposal]: "DuplicateActiveProposal",
};

/**
 * Typed error raised when a `governance` contract call fails with a contract
 * error code.
 */
export class GovernanceError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "GovernanceError",
      "governance",
      code,
      GOVERNANCE_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Oracle Adapter ────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `OracleError` in
 * `contracts/oracle-adapter/src/error.rs`.
 */
export enum OracleAdapterErrorCode {
  NotInitialized = 1,
  InvalidDecimals = 2,
  InvalidResolution = 3,
  UnknownAsset = 4,
  AssetAlreadyExists = 5,
  TooManyAssets = 6,
  NotReporter = 7,
  InvalidPrice = 8,
  TimestampInFuture = 9,
  StaleReport = 10,
  DeviationTooLarge = 11,
  InvalidDeviation = 12,
  Overflow = 13,
  ContractPaused = 14,
  NoPendingAdminTransfer = 15,
}

const ORACLE_ADAPTER_ERROR_NAMES: Readonly<Record<number, string>> = {
  [OracleAdapterErrorCode.NotInitialized]: "NotInitialized",
  [OracleAdapterErrorCode.InvalidDecimals]: "InvalidDecimals",
  [OracleAdapterErrorCode.InvalidResolution]: "InvalidResolution",
  [OracleAdapterErrorCode.UnknownAsset]: "UnknownAsset",
  [OracleAdapterErrorCode.AssetAlreadyExists]: "AssetAlreadyExists",
  [OracleAdapterErrorCode.TooManyAssets]: "TooManyAssets",
  [OracleAdapterErrorCode.NotReporter]: "NotReporter",
  [OracleAdapterErrorCode.InvalidPrice]: "InvalidPrice",
  [OracleAdapterErrorCode.TimestampInFuture]: "TimestampInFuture",
  [OracleAdapterErrorCode.StaleReport]: "StaleReport",
  [OracleAdapterErrorCode.DeviationTooLarge]: "DeviationTooLarge",
  [OracleAdapterErrorCode.InvalidDeviation]: "InvalidDeviation",
  [OracleAdapterErrorCode.Overflow]: "Overflow",
  [OracleAdapterErrorCode.ContractPaused]: "ContractPaused",
  [OracleAdapterErrorCode.NoPendingAdminTransfer]: "NoPendingAdminTransfer",
};

/**
 * Typed error raised when an `oracle-adapter` contract call fails with a
 * contract error code.
 */
export class OracleAdapterError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "OracleAdapterError",
      "oracle-adapter",
      code,
      ORACLE_ADAPTER_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Vault ─────────────────────────────────────────────────────────────────────

/**
 * Contract error discriminants matching `VaultError` in
 * `contracts/vault/src/error.rs`.
 */
export enum VaultContractErrorCode {
  NotInitialized = 1,
  InvalidExchangeRate = 2,
  InvalidMaxSupply = 3,
  InvalidLockup = 4,
  InvalidMetadata = 5,
  InvalidUnderlying = 6,
  ContractPaused = 7,
  InvalidAmount = 8,
  InsufficientBalance = 9,
  InsufficientAllowance = 10,
  Overflow = 11,
  InvalidExpiration = 12,
  NotCompliant = 13,
  SharesLocked = 14,
  ExceedsMaxSupply = 15,
  InvalidComplianceLevel = 16,
  NotMultipleOfExchangeRate = 17,
  OracleBaseMismatch = 18,
  PriceUnavailable = 19,
  InvalidPrice = 20,
  PriceInFuture = 21,
  StalePrice = 22,
  OracleDivergence = 23,
  IdenticalOracleSources = 24,
  OracleNotConfigured = 25,
  LedgerNotClosed = 26,
}

const VAULT_CONTRACT_ERROR_NAMES: Readonly<Record<number, string>> = {
  [VaultContractErrorCode.NotInitialized]: "NotInitialized",
  [VaultContractErrorCode.InvalidExchangeRate]: "InvalidExchangeRate",
  [VaultContractErrorCode.InvalidMaxSupply]: "InvalidMaxSupply",
  [VaultContractErrorCode.InvalidLockup]: "InvalidLockup",
  [VaultContractErrorCode.InvalidMetadata]: "InvalidMetadata",
  [VaultContractErrorCode.InvalidUnderlying]: "InvalidUnderlying",
  [VaultContractErrorCode.ContractPaused]: "ContractPaused",
  [VaultContractErrorCode.InvalidAmount]: "InvalidAmount",
  [VaultContractErrorCode.InsufficientBalance]: "InsufficientBalance",
  [VaultContractErrorCode.InsufficientAllowance]: "InsufficientAllowance",
  [VaultContractErrorCode.Overflow]: "Overflow",
  [VaultContractErrorCode.InvalidExpiration]: "InvalidExpiration",
  [VaultContractErrorCode.NotCompliant]: "NotCompliant",
  [VaultContractErrorCode.SharesLocked]: "SharesLocked",
  [VaultContractErrorCode.ExceedsMaxSupply]: "ExceedsMaxSupply",
  [VaultContractErrorCode.InvalidComplianceLevel]: "InvalidComplianceLevel",
  [VaultContractErrorCode.NotMultipleOfExchangeRate]: "NotMultipleOfExchangeRate",
  [VaultContractErrorCode.OracleBaseMismatch]: "OracleBaseMismatch",
  [VaultContractErrorCode.PriceUnavailable]: "PriceUnavailable",
  [VaultContractErrorCode.InvalidPrice]: "InvalidPrice",
  [VaultContractErrorCode.PriceInFuture]: "PriceInFuture",
  [VaultContractErrorCode.StalePrice]: "StalePrice",
  [VaultContractErrorCode.OracleDivergence]: "OracleDivergence",
  [VaultContractErrorCode.IdenticalOracleSources]: "IdenticalOracleSources",
  [VaultContractErrorCode.OracleNotConfigured]: "OracleNotConfigured",
  [VaultContractErrorCode.LedgerNotClosed]: "LedgerNotClosed",
};

/**
 * Typed error raised when a `vault` contract call fails with a contract error
 * code.
 */
export class VaultContractError extends ContractError {
  constructor(code: number, rawError?: string) {
    super(
      "VaultContractError",
      "vault",
      code,
      VAULT_CONTRACT_ERROR_NAMES[code] ?? "Unknown",
      rawError,
    );
  }
}

// ── Parsing & Dispatch ────────────────────────────────────────────────────────

const CONTRACT_ERROR_REGEX = /Error\(Contract,\s*#(\d+)\)/;

/**
 * Extracts the numeric Soroban contract error code from an RPC or host error
 * string (e.g. `"HostError: Error(Contract, #8)"` -> `8`).
 *
 * Returns `undefined` when the string does not contain a contract error code.
 */
export function parseContractErrorCode(errorMessage: string): number | undefined {
  const match = CONTRACT_ERROR_REGEX.exec(errorMessage);
  if (!match || match[1] === undefined) {
    return undefined;
  }
  const parsed = Number(match[1]);
  return Number.isSafeInteger(parsed) ? parsed : undefined;
}

/**
 * Maps a contract config key and numeric error code to the corresponding typed
 * {@link ContractError} subclass.
 */
export function mapContractError(
  configKey: string,
  code: number,
  rawError?: string,
): ContractError {
  switch (configKey) {
    case "rwaAsset":
      return new RwaAssetError(code, rawError);
    case "compliance":
      return new ComplianceError(code, rawError);
    case "registry":
      return new RegistryError(code, rawError);
    case "governance":
      return new GovernanceError(code, rawError);
    case "oracleAdapter":
      return new OracleAdapterError(code, rawError);
    case "vault":
      return new VaultContractError(code, rawError);
    default:
      return new ContractError("ContractError", configKey, code, "Unknown", rawError);
  }
}
