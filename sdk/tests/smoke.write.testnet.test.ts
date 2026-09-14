/**
 * Live write-path smoke test against a deployed `rwa-asset` over real Soroban RPC.
 *
 * Unlike `smoke.testnet.test.ts`, this one mints, moves and burns tokens and
 * spends fees. It therefore has its own opt-in, `SMOKE_WRITE_RWA_ASSET_ID`,
 * independent of the read smoke test's variables, and it refuses to run against
 * anything but the testnet passphrase. `CONTRIBUTING.md` lists what it needs and
 * what it changes.
 *
 * `client.test.ts` stubs `prepareTransaction`, `sendTransaction` and
 * `pollTransaction`, so only this file shows that a prepared footprint, the
 * resource fee, same-address auth and a cross-contract compliance `screen` all
 * hold up on the network.
 *
 * Each step asserts a before-and-after delta rather than an absolute value, so it
 * can be re-run against the same deployment without a reset. Two runs against the
 * same contract at the same time will disturb each other's deltas.
 */

import { describe, it, expect, beforeAll } from "vitest";
import { Account, Keypair, MuxedAccount, Networks, rpc, scValToNative } from "@stellar/stellar-sdk";

import { RwaAssetClient } from "../src/client.js";
import { DEFAULT_AUTH_VALIDITY_LEDGERS, authorizeEntries } from "../src/sponsored.js";
import { TESTNET_CONFIG } from "../src/types.js";
import { isValidContractId, isValidStellarAddress } from "../src/utils.js";
import type { StellarForgeConfig } from "../src/types.js";

// ─── Configuration ────────────────────────────────────────────────────────────

const ASSET_ID = process.env["SMOKE_WRITE_RWA_ASSET_ID"];
const ISSUER_SECRET = process.env["SMOKE_WRITE_ISSUER_SECRET"];
const SPENDER_SECRET = process.env["SMOKE_WRITE_SPENDER_SECRET"];
const RECIPIENT = process.env["SMOKE_WRITE_RECIPIENT_ADDRESS"];
/** Set when the asset screens transfers, so the rejection path is asserted too. */
const SCREENED = process.env["SMOKE_WRITE_SCREENED"] === "true";
/**
 * Set to also hand the asset's admin role from the issuer to the spender. That
 * is not something to do to a deployment anyone intends to keep, so it needs
 * its own opt-in.
 */
const TRANSFER_ADMIN = process.env["SMOKE_WRITE_TRANSFER_ADMIN"] === "true";

const RPC_URL = process.env["SOROBAN_RPC_URL"] ?? TESTNET_CONFIG.rpcUrl;
const NETWORK_PASSPHRASE =
  process.env["STELLAR_NETWORK_PASSPHRASE"] ?? TESTNET_CONFIG.networkPassphrase;

/**
 * A bare write can wait out its transaction's whole validity window plus a
 * grace period before it settles or gives up (see `awaitSettlement`).
 */
const WRITE_TIMEOUT = 300_000;

const MINT = 1_000n;
const TRANSFER = 250n;
const MUXED_TRANSFER = 100n;
const ALLOWANCE = 300n;
const SPEND = 120n;
const BURN_FROM = 30n;
const BURN = 50n;
const SPONSORED_TRANSFER = 40n;
const MUXED_ID = 42n;

/** How far ahead of the current ledger the smoke allowance stays spendable. */
const ALLOWANCE_LEDGERS = 10_000;

function configFor(signerSecret: string): StellarForgeConfig {
  return {
    network: "testnet",
    rpcUrl: RPC_URL,
    networkPassphrase: NETWORK_PASSPHRASE,
    contracts: { rwaAsset: ASSET_ID as string },
    signerSecret,
  };
}

// ─── Write path ───────────────────────────────────────────────────────────────

describe.runIf(ASSET_ID)("live RwaAssetClient writes", () => {
  let issuerClient: RwaAssetClient;
  let spenderClient: RwaAssetClient;
  let server: rpc.Server;
  let issuer: string;
  let spender: string;
  let recipient: string;

  interface Snapshot {
    supply: bigint;
    issuerBalance: bigint;
    recipientBalance: bigint;
    allowance: bigint;
  }

  async function snapshot(): Promise<Snapshot> {
    const [supply, issuerBalance, recipientBalance, allowance] = await Promise.all([
      issuerClient.totalSupply(),
      issuerClient.balance(issuer),
      issuerClient.balance(recipient),
      issuerClient.allowance(issuer, spender),
    ]);
    return { supply, issuerBalance, recipientBalance, allowance };
  }

  beforeAll(async () => {
    // Refused outright rather than trusted to configuration: these are real
    // transactions, and on mainnet they would spend real fees and mint real
    // tokens.
    if (NETWORK_PASSPHRASE !== Networks.TESTNET) {
      throw new Error(
        `The write smoke test only runs on testnet, but STELLAR_NETWORK_PASSPHRASE is "${NETWORK_PASSPHRASE}"`,
      );
    }
    if (!isValidContractId(ASSET_ID as string)) {
      throw new Error(`SMOKE_WRITE_RWA_ASSET_ID is not a valid contract id: ${ASSET_ID}`);
    }
    if (!ISSUER_SECRET || !SPENDER_SECRET || !RECIPIENT) {
      throw new Error(
        "SMOKE_WRITE_ISSUER_SECRET, SMOKE_WRITE_SPENDER_SECRET and SMOKE_WRITE_RECIPIENT_ADDRESS " +
          "are all required when SMOKE_WRITE_RWA_ASSET_ID is set",
      );
    }
    if (!isValidStellarAddress(RECIPIENT)) {
      throw new Error(`SMOKE_WRITE_RECIPIENT_ADDRESS is not a valid G-address: ${RECIPIENT}`);
    }
    if (!RPC_URL.startsWith("https://")) {
      throw new Error(`SOROBAN_RPC_URL must be https (the clients set allowHttp: false): ${RPC_URL}`);
    }

    issuer = Keypair.fromSecret(ISSUER_SECRET).publicKey();
    spender = Keypair.fromSecret(SPENDER_SECRET).publicKey();
    recipient = RECIPIENT;
    if (new Set([issuer, spender, recipient]).size !== 3) {
      throw new Error("The issuer, spender and recipient must be three different addresses");
    }

    issuerClient = new RwaAssetClient(configFor(ISSUER_SECRET));
    spenderClient = new RwaAssetClient(configFor(SPENDER_SECRET));
    server = new rpc.Server(RPC_URL);

    // Preconditions the admin has to set up. Failing here names the missing
    // step instead of surfacing as a contract error inside the first write.
    if (!(await issuerClient.isIssuer(issuer))) {
      throw new Error(`${issuer} does not hold the issuer role; grant it with set_issuer first`);
    }
    if (await issuerClient.isPaused()) {
      throw new Error("The asset is paused, so every write would fail with ContractPaused");
    }
  }, WRITE_TIMEOUT);

  it(
    "mint raises total supply and the holder's balance by the amount",
    async () => {
      const before = await snapshot();

      const { hash, ledger } = await issuerClient.mint(issuer, issuer, MINT);
      expect(hash).toMatch(/^[0-9a-f]{64}$/);
      expect(ledger).toBeGreaterThan(0);

      const after = await snapshot();
      expect(after.supply - before.supply).toBe(MINT);
      expect(after.issuerBalance - before.issuerBalance).toBe(MINT);
    },
    WRITE_TIMEOUT,
  );

  it(
    "transfer moves the amount between the two balances and leaves supply alone",
    async () => {
      const before = await snapshot();

      await issuerClient.transfer(issuer, recipient, TRANSFER);

      const after = await snapshot();
      expect(before.issuerBalance - after.issuerBalance).toBe(TRANSFER);
      expect(after.recipientBalance - before.recipientBalance).toBe(TRANSFER);
      expect(after.supply).toBe(before.supply);
    },
    WRITE_TIMEOUT,
  );

  // SEP-41: a muxed destination credits the underlying address, and its id
  // travels in the event for the recipient's own bookkeeping.
  it(
    "transfer to a muxed address credits the base account and reports the muxed id",
    async () => {
      const muxed = new MuxedAccount(new Account(recipient, "0"), MUXED_ID.toString()).accountId();
      const before = await snapshot();

      const { hash, ledger } = await issuerClient.transfer(issuer, muxed, MUXED_TRANSFER);

      const after = await snapshot();
      expect(after.recipientBalance - before.recipientBalance).toBe(MUXED_TRANSFER);

      const { events } = await server.getEvents({
        startLedger: ledger,
        filters: [{ type: "contract", contractIds: [ASSET_ID as string] }],
        limit: 100,
      });
      const transfer = events.find(
        (e) => e.txHash === hash && e.topic[0] !== undefined && scValToNative(e.topic[0]) === "transfer",
      );
      expect(transfer, "no transfer event was found for the muxed transfer").toBeDefined();

      const topicTo = transfer?.topic[2];
      expect(topicTo && scValToNative(topicTo)).toBe(recipient);
      const data = scValToNative(transfer!.value) as { amount: bigint; to_muxed_id: bigint | null };
      expect(data.amount).toBe(MUXED_TRANSFER);
      expect(data.to_muxed_id).toBe(MUXED_ID);
    },
    WRITE_TIMEOUT,
  );

  // Only meaningful when the asset has a compliance contract configured. The
  // stranger holds no KYC record, so screening must refuse it during simulation,
  // before anything is signed.
  it.runIf(SCREENED)(
    "refuses a transfer to an address with no verification record",
    async () => {
      const stranger = Keypair.random().publicKey();
      await expect(issuerClient.buildTransferTx(issuer, stranger, 1n)).rejects.toThrow(
        /Error\(Contract, #11\)/,
      );
    },
    WRITE_TIMEOUT,
  );

  it(
    "approve sets an allowance that reads back until its expiry ledger",
    async () => {
      const { sequence } = await server.getLatestLedger();

      await issuerClient.approve(issuer, spender, ALLOWANCE, sequence + ALLOWANCE_LEDGERS);

      // approve replaces rather than adds, so this is an absolute check and
      // still re-runnable.
      expect(await issuerClient.allowance(issuer, spender)).toBe(ALLOWANCE);
    },
    WRITE_TIMEOUT,
  );

  it(
    "transferFrom moves the owner's tokens and spends the spender's allowance",
    async () => {
      const before = await snapshot();

      await spenderClient.transferFrom(spender, issuer, recipient, SPEND);

      const after = await snapshot();
      expect(before.issuerBalance - after.issuerBalance).toBe(SPEND);
      expect(after.recipientBalance - before.recipientBalance).toBe(SPEND);
      expect(before.allowance - after.allowance).toBe(SPEND);
      expect(after.supply).toBe(before.supply);
    },
    WRITE_TIMEOUT,
  );

  it(
    "burnFrom destroys the owner's tokens and spends the spender's allowance",
    async () => {
      const before = await snapshot();

      await spenderClient.burnFrom(spender, issuer, BURN_FROM);

      const after = await snapshot();
      expect(before.supply - after.supply).toBe(BURN_FROM);
      expect(before.issuerBalance - after.issuerBalance).toBe(BURN_FROM);
      expect(before.allowance - after.allowance).toBe(BURN_FROM);
    },
    WRITE_TIMEOUT,
  );

  it(
    "burn lowers total supply and the holder's balance by the amount",
    async () => {
      const before = await snapshot();

      await issuerClient.burn(issuer, BURN);

      const after = await snapshot();
      expect(before.supply - after.supply).toBe(BURN);
      expect(before.issuerBalance - after.issuerBalance).toBe(BURN);
    },
    WRITE_TIMEOUT,
  );

  // #31: one account pays for a write another authorizes. The issuer signs an
  // authorization entry and the spender sources the transaction, so the
  // issuer's sequence number must not move. With screening on, the transfer
  // still goes through the compliance contract.
  it(
    "sponsored transfer: the issuer authorizes while the spender pays",
    async () => {
      const before = await snapshot();
      const issuerSequence = (await server.getAccount(issuer)).sequenceNumber();

      const sponsored = await spenderClient.buildSponsoredTransferTx(
        issuer,
        recipient,
        SPONSORED_TRANSFER,
        { feeSource: spender },
      );
      expect(sponsored.transaction.source).toBe(spender);
      expect(sponsored.authorizers).toEqual([issuer]);

      const { sequence } = await server.getLatestLedger();
      const signed = await authorizeEntries(
        sponsored.authEntries,
        Keypair.fromSecret(ISSUER_SECRET as string),
        sequence + DEFAULT_AUTH_VALIDITY_LEDGERS,
        NETWORK_PASSPHRASE,
      );
      const { hash } = await spenderClient.submitSponsoredTx(sponsored, signed);
      expect(hash).toMatch(/^[0-9a-f]{64}$/);

      const after = await snapshot();
      expect(before.issuerBalance - after.issuerBalance).toBe(SPONSORED_TRANSFER);
      expect(after.recipientBalance - before.recipientBalance).toBe(SPONSORED_TRANSFER);
      expect((await server.getAccount(issuer)).sequenceNumber()).toBe(issuerSequence);
    },
    WRITE_TIMEOUT,
  );

  // Runs last: it hands the admin role away. The contract needs the current and
  // the incoming admin to authorize, so the current admin pays and the incoming
  // admin signs its entry.
  it.runIf(TRANSFER_ADMIN)(
    "transferAdmin: the current admin pays while the incoming admin signs",
    async () => {
      if ((await issuerClient.admin()) !== issuer) {
        throw new Error("SMOKE_WRITE_TRANSFER_ADMIN needs the issuer to be the asset's admin");
      }

      const sponsored = await issuerClient.buildTransferAdminTx(issuer, spender);
      expect(sponsored.transaction.source).toBe(issuer);
      expect(sponsored.authorizers.filter((a) => a !== null)).toEqual([spender]);

      const { sequence } = await server.getLatestLedger();
      const signed = await authorizeEntries(
        sponsored.authEntries,
        Keypair.fromSecret(SPENDER_SECRET as string),
        sequence + DEFAULT_AUTH_VALIDITY_LEDGERS,
        NETWORK_PASSPHRASE,
      );
      await issuerClient.submitSponsoredTx(sponsored, signed);

      expect(await issuerClient.admin()).toBe(spender);
    },
    WRITE_TIMEOUT,
  );
});
