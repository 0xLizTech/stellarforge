import { Account, Keypair, rpc } from "@stellar/stellar-sdk";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  ComplianceClient,
  ComplianceError,
  ComplianceErrorCode,
  ContractError,
  GovernanceClient,
  GovernanceError,
  GovernanceErrorCode,
  mapContractError,
  OracleAdapterClient,
  OracleAdapterError,
  OracleAdapterErrorCode,
  parseContractErrorCode,
  RegistryClient,
  RegistryError,
  RegistryErrorCode,
  RwaAssetClient,
  RwaAssetError,
  RwaAssetErrorCode,
  TESTNET_CONFIG,
  VaultClient,
  VaultContractError,
  VaultContractErrorCode,
} from "../src/index.js";
import { fails, stubSimulation } from "./helpers.js";

const HOLDER = Keypair.random().publicKey();
const CONTRACT_ID = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";

const config = {
  ...TESTNET_CONFIG,
  contracts: {
    rwaAsset: CONTRACT_ID,
    compliance: CONTRACT_ID,
    registry: CONTRACT_ID,
    governance: CONTRACT_ID,
    oracleAdapter: CONTRACT_ID,
    vault: CONTRACT_ID,
  },
};

afterEach(() => {
  vi.restoreAllMocks();
});

describe("parseContractErrorCode", () => {
  it("extracts the numeric code from Soroban HostError strings", () => {
    expect(parseContractErrorCode("HostError: Error(Contract, #8)")).toBe(8);
    expect(parseContractErrorCode("Simulation error: HostError: Error(Contract, #25)")).toBe(25);
    expect(parseContractErrorCode("Error(Contract,#1)")).toBe(1);
  });

  it("returns undefined for non-contract errors", () => {
    expect(parseContractErrorCode("HostError: Error(Auth, InvalidAction)")).toBeUndefined();
    expect(parseContractErrorCode("Network timeout")).toBeUndefined();
    expect(parseContractErrorCode("")).toBeUndefined();
  });
});

describe("RwaAssetError", () => {
  const cases: Array<[number, string]> = [
    [RwaAssetErrorCode.AlreadyInitialized, "AlreadyInitialized"],
    [RwaAssetErrorCode.NotInitialized, "NotInitialized"],
    [RwaAssetErrorCode.NotIssuer, "NotIssuer"],
    [RwaAssetErrorCode.InvalidAmount, "InvalidAmount"],
    [RwaAssetErrorCode.InsufficientBalance, "InsufficientBalance"],
    [RwaAssetErrorCode.InsufficientAllowance, "InsufficientAllowance"],
    [RwaAssetErrorCode.ExceedsMaxSupply, "ExceedsMaxSupply"],
    [RwaAssetErrorCode.ContractPaused, "ContractPaused"],
    [RwaAssetErrorCode.Overflow, "Overflow"],
    [RwaAssetErrorCode.InvalidMetadata, "InvalidMetadata"],
    [RwaAssetErrorCode.NotCompliant, "NotCompliant"],
    [RwaAssetErrorCode.DecimalsImmutable, "DecimalsImmutable"],
    [RwaAssetErrorCode.InvalidSupplyCap, "InvalidSupplyCap"],
    [RwaAssetErrorCode.InvalidExpiration, "InvalidExpiration"],
    [RwaAssetErrorCode.InvalidComplianceLevel, "InvalidComplianceLevel"],
    [RwaAssetErrorCode.InvalidLegalDocHash, "InvalidLegalDocHash"],
    [RwaAssetErrorCode.GovernanceNotConfigured, "GovernanceNotConfigured"],
    [RwaAssetErrorCode.NoPendingCapIncrease, "NoPendingCapIncrease"],
    [RwaAssetErrorCode.NoPendingAdminTransfer, "NoPendingAdminTransfer"],
    [RwaAssetErrorCode.NotForcedTransferRole, "NotForcedTransferRole"],
    [RwaAssetErrorCode.InvalidReasonHash, "InvalidReasonHash"],
    [RwaAssetErrorCode.InvalidForcedTransferParty, "InvalidForcedTransferParty"],
    [RwaAssetErrorCode.AddressFrozen, "AddressFrozen"],
    [RwaAssetErrorCode.AddressLocked, "AddressLocked"],
    [RwaAssetErrorCode.ExceedsHolderCap, "ExceedsHolderCap"],
  ];

  it("maps every RwaError code to its readable name and preserves the numeric code", () => {
    for (const [code, expectedName] of cases) {
      const err = new RwaAssetError(code);
      expect(err).toBeInstanceOf(Error);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(RwaAssetError);
      expect(err.name).toBe("RwaAssetError");
      expect(err.contract).toBe("rwa-asset");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("handles unknown codes gracefully while preserving the numeric code", () => {
    const err = new RwaAssetError(999);
    expect(err.code).toBe(999);
    expect(err.codeName).toBe("Unknown");
  });

  it("throws RwaAssetError from RwaAssetClient on contract error during read", async () => {
    stubSimulation(fails("HostError: Error(Contract, #8)"));
    try {
      await new RwaAssetClient(config).balance(HOLDER);
      expect.unreachable("expected RwaAssetError");
    } catch (err) {
      expect(err).toBeInstanceOf(RwaAssetError);
      expect((err as RwaAssetError).code).toBe(8);
      expect((err as RwaAssetError).codeName).toBe("ContractPaused");
    }
  });

  it("throws RwaAssetError from RwaAssetClient on contract error during write", async () => {
    vi.spyOn(rpc.Server.prototype, "getAccount").mockResolvedValue(new Account(HOLDER, "1"));
    vi.spyOn(rpc.Server.prototype, "prepareTransaction").mockRejectedValue(
      new Error("HostError: Error(Contract, #11)"),
    );
    try {
      await new RwaAssetClient(config).buildTransferTx(HOLDER, HOLDER, 10n);
      expect.unreachable("expected RwaAssetError");
    } catch (err) {
      expect(err).toBeInstanceOf(RwaAssetError);
      expect((err as RwaAssetError).code).toBe(11);
      expect((err as RwaAssetError).codeName).toBe("NotCompliant");
    }
  });
});

describe("ComplianceError", () => {
  const cases: Array<[number, string]> = [
    [ComplianceErrorCode.AlreadyInitialized, "AlreadyInitialized"],
    [ComplianceErrorCode.NotInitialized, "NotInitialized"],
    [ComplianceErrorCode.InvalidLevel, "InvalidLevel"],
    [ComplianceErrorCode.AlreadyExpired, "AlreadyExpired"],
    [ComplianceErrorCode.InvalidJurisdiction, "InvalidJurisdiction"],
    [ComplianceErrorCode.ContractPaused, "ContractPaused"],
    [ComplianceErrorCode.BatchTooLarge, "BatchTooLarge"],
    [ComplianceErrorCode.NoPendingAdminTransfer, "NoPendingAdminTransfer"],
  ];

  it("maps every ComplianceError code to its readable name", () => {
    for (const [code, expectedName] of cases) {
      const err = new ComplianceError(code);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(ComplianceError);
      expect(err.contract).toBe("compliance");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("throws ComplianceError from ComplianceClient on simulation failure", async () => {
    stubSimulation(fails("HostError: Error(Contract, #5)"));
    try {
      await new ComplianceClient(config).isCompliant(HOLDER, 1);
      expect.unreachable("expected ComplianceError");
    } catch (err) {
      expect(err).toBeInstanceOf(ComplianceError);
      expect((err as ComplianceError).code).toBe(5);
      expect((err as ComplianceError).codeName).toBe("InvalidJurisdiction");
    }
  });
});

describe("RegistryError", () => {
  const cases: Array<[number, string]> = [
    [RegistryErrorCode.AlreadyInitialized, "AlreadyInitialized"],
    [RegistryErrorCode.NotInitialized, "NotInitialized"],
    [RegistryErrorCode.AssetNotFound, "AssetNotFound"],
    [RegistryErrorCode.PageTooLarge, "PageTooLarge"],
    [RegistryErrorCode.Overflow, "Overflow"],
    [RegistryErrorCode.ContractPaused, "ContractPaused"],
    [RegistryErrorCode.NoPendingAdminTransfer, "NoPendingAdminTransfer"],
  ];

  it("maps every RegistryError code to its readable name", () => {
    for (const [code, expectedName] of cases) {
      const err = new RegistryError(code);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(RegistryError);
      expect(err.contract).toBe("registry");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("throws RegistryError from RegistryClient on simulation failure", async () => {
    stubSimulation(fails("HostError: Error(Contract, #3)"));
    try {
      await new RegistryClient(config).getAsset(CONTRACT_ID);
      expect.unreachable("expected RegistryError");
    } catch (err) {
      expect(err).toBeInstanceOf(RegistryError);
      expect((err as RegistryError).code).toBe(3);
      expect((err as RegistryError).codeName).toBe("AssetNotFound");
    }
  });
});

describe("GovernanceError", () => {
  const cases: Array<[number, string]> = [
    [GovernanceErrorCode.AlreadyInitialized, "AlreadyInitialized"],
    [GovernanceErrorCode.NotInitialized, "NotInitialized"],
    [GovernanceErrorCode.ProposalNotFound, "ProposalNotFound"],
    [GovernanceErrorCode.AlreadyVoted, "AlreadyVoted"],
    [GovernanceErrorCode.ProposalNotActive, "ProposalNotActive"],
    [GovernanceErrorCode.VotingClosed, "VotingClosed"],
    [GovernanceErrorCode.VotingNotClosed, "VotingNotClosed"],
    [GovernanceErrorCode.InvalidWeight, "InvalidWeight"],
    [GovernanceErrorCode.Overflow, "Overflow"],
    [GovernanceErrorCode.InvalidVotingPeriod, "InvalidVotingPeriod"],
    [GovernanceErrorCode.TitleTooLong, "TitleTooLong"],
    [GovernanceErrorCode.NotAuthorizedToCancel, "NotAuthorizedToCancel"],
    [GovernanceErrorCode.ProposalAlreadyHasVotes, "ProposalAlreadyHasVotes"],
    [GovernanceErrorCode.PageTooLarge, "PageTooLarge"],
    [GovernanceErrorCode.InvalidDepositAmount, "InvalidDepositAmount"],
    [GovernanceErrorCode.NoPendingAdminTransfer, "NoPendingAdminTransfer"],
    [GovernanceErrorCode.InvalidQuorum, "InvalidQuorum"],
    [GovernanceErrorCode.InvalidPassThresholdBps, "InvalidPassThresholdBps"],
    [GovernanceErrorCode.DuplicateActiveProposal, "DuplicateActiveProposal"],
  ];

  it("maps every GovernanceError code to its readable name", () => {
    for (const [code, expectedName] of cases) {
      const err = new GovernanceError(code);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(GovernanceError);
      expect(err.contract).toBe("governance");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("throws GovernanceError from GovernanceClient on simulation failure", async () => {
    stubSimulation(fails("HostError: Error(Contract, #19)"));
    try {
      await new GovernanceClient(config).getProposal(1n);
      expect.unreachable("expected GovernanceError");
    } catch (err) {
      expect(err).toBeInstanceOf(GovernanceError);
      expect((err as GovernanceError).code).toBe(19);
      expect((err as GovernanceError).codeName).toBe("DuplicateActiveProposal");
    }
  });
});

describe("OracleAdapterError", () => {
  const cases: Array<[number, string]> = [
    [OracleAdapterErrorCode.NotInitialized, "NotInitialized"],
    [OracleAdapterErrorCode.InvalidDecimals, "InvalidDecimals"],
    [OracleAdapterErrorCode.InvalidResolution, "InvalidResolution"],
    [OracleAdapterErrorCode.UnknownAsset, "UnknownAsset"],
    [OracleAdapterErrorCode.AssetAlreadyExists, "AssetAlreadyExists"],
    [OracleAdapterErrorCode.TooManyAssets, "TooManyAssets"],
    [OracleAdapterErrorCode.NotReporter, "NotReporter"],
    [OracleAdapterErrorCode.InvalidPrice, "InvalidPrice"],
    [OracleAdapterErrorCode.TimestampInFuture, "TimestampInFuture"],
    [OracleAdapterErrorCode.StaleReport, "StaleReport"],
    [OracleAdapterErrorCode.DeviationTooLarge, "DeviationTooLarge"],
    [OracleAdapterErrorCode.InvalidDeviation, "InvalidDeviation"],
    [OracleAdapterErrorCode.Overflow, "Overflow"],
    [OracleAdapterErrorCode.ContractPaused, "ContractPaused"],
    [OracleAdapterErrorCode.NoPendingAdminTransfer, "NoPendingAdminTransfer"],
  ];

  it("maps every OracleError code to its readable name", () => {
    for (const [code, expectedName] of cases) {
      const err = new OracleAdapterError(code);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(OracleAdapterError);
      expect(err.contract).toBe("oracle-adapter");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("throws OracleAdapterError from OracleAdapterClient on simulation failure", async () => {
    stubSimulation(fails("HostError: Error(Contract, #11)"));
    try {
      await new OracleAdapterClient(config).decimals();
      expect.unreachable("expected OracleAdapterError");
    } catch (err) {
      expect(err).toBeInstanceOf(OracleAdapterError);
      expect((err as OracleAdapterError).code).toBe(11);
      expect((err as OracleAdapterError).codeName).toBe("DeviationTooLarge");
    }
  });
});

describe("VaultContractError", () => {
  const cases: Array<[number, string]> = [
    [VaultContractErrorCode.NotInitialized, "NotInitialized"],
    [VaultContractErrorCode.InvalidExchangeRate, "InvalidExchangeRate"],
    [VaultContractErrorCode.InvalidMaxSupply, "InvalidMaxSupply"],
    [VaultContractErrorCode.InvalidLockup, "InvalidLockup"],
    [VaultContractErrorCode.InvalidMetadata, "InvalidMetadata"],
    [VaultContractErrorCode.InvalidUnderlying, "InvalidUnderlying"],
    [VaultContractErrorCode.ContractPaused, "ContractPaused"],
    [VaultContractErrorCode.InvalidAmount, "InvalidAmount"],
    [VaultContractErrorCode.InsufficientBalance, "InsufficientBalance"],
    [VaultContractErrorCode.InsufficientAllowance, "InsufficientAllowance"],
    [VaultContractErrorCode.Overflow, "Overflow"],
    [VaultContractErrorCode.InvalidExpiration, "InvalidExpiration"],
    [VaultContractErrorCode.NotCompliant, "NotCompliant"],
    [VaultContractErrorCode.SharesLocked, "SharesLocked"],
    [VaultContractErrorCode.ExceedsMaxSupply, "ExceedsMaxSupply"],
    [VaultContractErrorCode.InvalidComplianceLevel, "InvalidComplianceLevel"],
    [VaultContractErrorCode.NotMultipleOfExchangeRate, "NotMultipleOfExchangeRate"],
    [VaultContractErrorCode.OracleBaseMismatch, "OracleBaseMismatch"],
    [VaultContractErrorCode.PriceUnavailable, "PriceUnavailable"],
    [VaultContractErrorCode.InvalidPrice, "InvalidPrice"],
    [VaultContractErrorCode.PriceInFuture, "PriceInFuture"],
    [VaultContractErrorCode.StalePrice, "StalePrice"],
    [VaultContractErrorCode.OracleDivergence, "OracleDivergence"],
    [VaultContractErrorCode.IdenticalOracleSources, "IdenticalOracleSources"],
    [VaultContractErrorCode.OracleNotConfigured, "OracleNotConfigured"],
    [VaultContractErrorCode.LedgerNotClosed, "LedgerNotClosed"],
  ];

  it("maps every VaultError code to its readable name", () => {
    for (const [code, expectedName] of cases) {
      const err = new VaultContractError(code);
      expect(err).toBeInstanceOf(ContractError);
      expect(err).toBeInstanceOf(VaultContractError);
      expect(err.contract).toBe("vault");
      expect(err.code).toBe(code);
      expect(err.codeName).toBe(expectedName);
    }
  });

  it("throws VaultContractError from VaultClient on simulation failure", async () => {
    stubSimulation(fails("HostError: Error(Contract, #14)"));
    try {
      await new VaultClient(config).balance(HOLDER);
      expect.unreachable("expected VaultContractError");
    } catch (err) {
      expect(err).toBeInstanceOf(VaultContractError);
      expect((err as VaultContractError).code).toBe(14);
      expect((err as VaultContractError).codeName).toBe("SharesLocked");
    }
  });
});

describe("mapContractError fallback and non-contract errors", () => {
  it("falls back to base ContractError for an unrecognized contract key", () => {
    const err = mapContractError("customContract", 42, "raw");
    expect(err).toBeInstanceOf(ContractError);
    expect(err.contract).toBe("customContract");
    expect(err.code).toBe(42);
    expect(err.codeName).toBe("Unknown");
    expect(err.rawError).toBe("raw");
  });

  it("throws a plain Error when simulation fails without a contract error code", async () => {
    stubSimulation(fails("HostError: Error(Auth, InvalidAction)"));
    try {
      await new RwaAssetClient(config).totalSupply();
      expect.unreachable("expected Error");
    } catch (err) {
      expect(err).toBeInstanceOf(Error);
      expect(err).not.toBeInstanceOf(ContractError);
      expect((err as Error).message).toBe(
        "Simulation error: HostError: Error(Auth, InvalidAction)",
      );
    }
  });
});
