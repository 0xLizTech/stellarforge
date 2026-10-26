import { Keypair } from "@stellar/stellar-sdk";
import {
  OracleAdapterClient,
  checkDeviation,
  tickOf,
} from "@stellarforge-protocol/sdk";
import type { OracleAsset, PriceData } from "@stellarforge-protocol/sdk";
import { verifyAppraisalSignature } from "./canonical.js";
import { ExitCode, type ReportingResult, type SignedAppraisalFile } from "./types.js";

export interface ReporterConfig {
  oracleClient: OracleAdapterClient;
  reporterKeypair: Keypair;
  relayerKeypair?: Keypair;
  expectedAppraiserPublicKey?: string;
  dryRun?: boolean;
}

/**
 * Converts a decimal string to an integer scaled by `10^decimals` without floating-point inaccuracies.
 * Refuses conversion if more fractional digits are supplied than the contract's decimals scale.
 */
export function parseExactPrice(priceStr: string, decimals: number): bigint {
  const parts = priceStr.trim().split(".");
  if (parts.length > 2) {
    throw new Error(`Invalid price string format: ${priceStr}`);
  }

  const wholeStr = parts[0] || "0";
  const fracStr = parts[1] || "";

  if (fracStr.length > decimals) {
    throw new Error(
      `Lossy price conversion: ${priceStr} has ${fracStr.length} decimal places, but oracle resolution allows at most ${decimals}`
    );
  }

  const whole = BigInt(wholeStr);
  const paddedFrac = fracStr.padEnd(decimals, "0");
  const frac = BigInt(paddedFrac);

  return whole * 10n ** BigInt(decimals) + frac;
}

/**
 * Executes the full validation and submission flow for a signed appraisal.
 */
export async function submitSignedAppraisal(
  signedFile: SignedAppraisalFile,
  config: ReporterConfig
): Promise<ReportingResult> {
  // 1. Signature Verification
  if (!verifyAppraisalSignature(signedFile, config.expectedAppraiserPublicKey)) {
    return {
      success: false,
      exitCode: ExitCode.AuthFailure,
      action: "rejected",
      error: "Authentication failed: invalid or unverified Ed25519 signature on appraisal payload",
    };
  }

  const { payload } = signedFile;
  const client = config.oracleClient;

  try {
    // 2. Pre-flight Reads
    const [oracleDecimals, resolution, isReporter, lastPriceData, assetConfig] =
      await Promise.all([
        client.decimals(),
        client.resolution(),
        client.isReporter(payload.asset, config.reporterKeypair.publicKey()),
        client.lastprice(payload.asset),
        client.assetConfig(payload.asset),
      ]);

    // Check reporter authorization
    if (!isReporter) {
      return {
        success: false,
        exitCode: ExitCode.PreflightFailure,
        action: "rejected",
        error: `Reporter ${config.reporterKeypair.publicKey()} is not authorized for the specified asset`,
      };
    }

    // Convert price to exact integer
    let scaledPrice: bigint;
    try {
      scaledPrice = parseExactPrice(payload.price, oracleDecimals);
    } catch (err) {
      return {
        success: false,
        exitCode: ExitCode.PreflightFailure,
        action: "rejected",
        error: err instanceof Error ? err.message : String(err),
      };
    }

    if (scaledPrice <= 0n) {
      return {
        success: false,
        exitCode: ExitCode.PreflightFailure,
        action: "rejected",
        error: `Price must be strictly positive, got ${payload.price}`,
      };
    }

    // Validate timestamp and tick
    const observedDate = new Date(payload.observedAt);
    const observedTimestamp = BigInt(Math.floor(observedDate.getTime() / 1000));
    const nowSecs = BigInt(Math.floor(Date.now() / 1000));

    if (observedTimestamp > nowSecs) {
      return {
        success: false,
        exitCode: ExitCode.PreflightFailure,
        action: "rejected",
        error: `Appraisal timestamp ${payload.observedAt} is in the future`,
      };
    }

    const currentTick = tickOf(observedTimestamp, resolution);

    if (lastPriceData) {
      const lastTick = tickOf(lastPriceData.timestamp, resolution);
      if (currentTick <= lastTick) {
        // Check if identical price already recorded at this tick (Idempotency)
        if (currentTick === lastTick && lastPriceData.price === scaledPrice) {
          return {
            success: true,
            exitCode: ExitCode.Success,
            action: "idempotent_duplicate",
            tick: currentTick.toString(),
            details: {
              message: "Price for this resolution tick is already recorded identically on-chain",
            },
          };
        }

        if (currentTick === lastTick && lastPriceData.price !== scaledPrice) {
          return {
            success: false,
            exitCode: ExitCode.PreflightFailure,
            action: "rejected",
            error: `Conflicting price report at tick ${currentTick}: current on-chain price is ${lastPriceData.price}, submitted is ${scaledPrice}`,
          };
        }

        return {
          success: false,
          exitCode: ExitCode.PreflightFailure,
          action: "rejected",
          error: `Stale report: tick ${currentTick} is not later than latest tick ${lastTick}`,
        };
      }

      // Check deviation limits
      if (assetConfig && assetConfig.maxDeviationBps > 0) {
        try {
          checkDeviation(lastPriceData.price, scaledPrice, assetConfig.maxDeviationBps);
        } catch (err) {
          return {
            success: false,
            exitCode: ExitCode.PreflightFailure,
            action: "rejected",
            error: `Deviation limit exceeded: ${err instanceof Error ? err.message : String(err)} (admin override required)`,
          };
        }
      }
    }

    // 3. Dry Run Check
    if (config.dryRun) {
      return {
        success: true,
        exitCode: ExitCode.Success,
        action: "dry_run",
        tick: currentTick.toString(),
        details: {
          asset: payload.asset,
          scaledPrice: scaledPrice.toString(),
          timestamp: observedTimestamp.toString(),
        },
      };
    }

    // 4. Submission (Sponsored Mode)
    const relayer = config.relayerKeypair || config.reporterKeypair;
    const sponsored = await client.buildSponsoredReportTx(
      config.reporterKeypair.publicKey(),
      payload.asset,
      scaledPrice,
      observedTimestamp,
      { feeSource: relayer.publicKey() }
    );

    // Sign reporter auth entries
    const signedTx = client.authorizeEntries(sponsored, config.reporterKeypair);
    const finalized = client.finalizeSponsoredTx(signedTx);

    // Relayer signs and submits fee payment
    finalized.sign(relayer);

    const txResult = await client.submitSponsoredTx(finalized);
    return {
      success: true,
      exitCode: ExitCode.Success,
      action: "submitted",
      txHash: txResult.hash,
      ledger: txResult.ledger,
      tick: currentTick.toString(),
    };
  } catch (err: any) {
    // 5. Handle Unknown Outcomes / Network Drops with On-Chain Parity Check
    if (err && err.name === "TransactionOutcomeUnknownError") {
      try {
        const resolution = await client.resolution();
        const observedDate = new Date(payload.observedAt);
        const observedTimestamp = BigInt(Math.floor(observedDate.getTime() / 1000));
        const currentTick = tickOf(observedTimestamp, resolution);

        const onChainPrice = await client.price(payload.asset, currentTick);
        const oracleDecimals = await client.decimals();
        const scaledPrice = parseExactPrice(payload.price, oracleDecimals);

        if (onChainPrice && onChainPrice.price === scaledPrice) {
          return {
            success: true,
            exitCode: ExitCode.Success,
            action: "submitted",
            tick: currentTick.toString(),
            details: {
              recoveredFromUnknownOutcome: true,
            },
          };
        }
      } catch {
        // Fall through to UnknownOutcome exit code
      }

      return {
        success: false,
        exitCode: ExitCode.UnknownOutcome,
        action: "rejected",
        error: "Transaction submission timed out or returned unknown outcome; report could not be verified on-chain",
      };
    }

    return {
      success: false,
      exitCode: ExitCode.SubmissionFailure,
      action: "rejected",
      error: `Submission failed: ${err instanceof Error ? err.message : String(err)}`,
    };
  }
}
