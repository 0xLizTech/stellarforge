import { describe, it, expect } from "vitest";
import { Account, Keypair, MuxedAccount } from "@stellar/stellar-sdk";

import { RwaAssetClient } from "../src/client.js";
import type { Sep41TokenClient } from "../src/client.js";
import type { StellarForgeConfig } from "../src/types.js";
import {
  RWA_ID,
  HOLDER,
  SPENDER,
  SIGNER as ISSUER,
  SIGNER_SECRET as ISSUER_SECRET,
  SUBMITTED_HASH,
  configWith,
  stubSimulation,
  stubWritePath,
  succeeds,
  invocation,
  builtInvocation,
  i128,
  str,
  u32,
} from "./helpers.js";

describe("SEP-41 Token Interface Compatibility (#180)", () => {
  const rwaConfig: StellarForgeConfig = configWith({ rwaAsset: RWA_ID });
  const signerConfig: StellarForgeConfig = { ...rwaConfig, signerSecret: ISSUER_SECRET };

  it("satisfies the Sep41TokenClient interface contract", () => {
    const client: Sep41TokenClient = new RwaAssetClient(rwaConfig);

    // Verify all 10 SEP-41 methods are implemented on the client instance
    expect(typeof client.balance).toBe("function");
    expect(typeof client.allowance).toBe("function");
    expect(typeof client.approve).toBe("function");
    expect(typeof client.transfer).toBe("function");
    expect(typeof client.transferFrom).toBe("function");
    expect(typeof client.burn).toBe("function");
    expect(typeof client.burnFrom).toBe("function");
    expect(typeof client.decimals).toBe("function");
    expect(typeof client.name).toBe("function");
    expect(typeof client.symbol).toBe("function");
  });

  describe("SEP-41 read queries", () => {
    it("decimals(): returns the asset decimals (u32)", async () => {
      const spy = stubSimulation(succeeds(u32(7)));
      const client = new RwaAssetClient(rwaConfig);
      const dec = await client.decimals();
      expect(dec).toBe(7);
      expect(invocation(spy)).toEqual({ fn: "decimals", args: [] });
    });

    it("name(): returns the asset name (String)", async () => {
      const spy = stubSimulation(succeeds(str("Treasury Bill Token")));
      const client = new RwaAssetClient(rwaConfig);
      const name = await client.name();
      expect(name).toBe("Treasury Bill Token");
      expect(invocation(spy)).toEqual({ fn: "name", args: [] });
    });

    it("symbol(): returns the asset symbol (String)", async () => {
      const spy = stubSimulation(succeeds(str("USTB-2026")));
      const client = new RwaAssetClient(rwaConfig);
      const sym = await client.symbol();
      expect(sym).toBe("USTB-2026");
      expect(invocation(spy)).toEqual({ fn: "symbol", args: [] });
    });

    it("balance(id): returns the account balance (i128)", async () => {
      const spy = stubSimulation(succeeds(i128(50_000_000n)));
      const client = new RwaAssetClient(rwaConfig);
      const bal = await client.balance(HOLDER);
      expect(bal).toBe(50_000_000n);
      expect(invocation(spy)).toEqual({ fn: "balance", args: [HOLDER] });
    });

    it("allowance(from, spender): returns the spender allowance (i128)", async () => {
      const spy = stubSimulation(succeeds(i128(10_000_000n)));
      const client = new RwaAssetClient(rwaConfig);
      const allow = await client.allowance(HOLDER, SPENDER);
      expect(allow).toBe(10_000_000n);
      expect(invocation(spy)).toEqual({ fn: "allowance", args: [HOLDER, SPENDER] });
    });
  });

  describe("SEP-41 write operations (bare submission & builder)", () => {
    it("approve: builds and submits SEP-41 allowance approval", async () => {
      stubWritePath();
      const client = new RwaAssetClient(signerConfig);
      const res = await client.approve(ISSUER, SPENDER, 1_000_000n, 10_000);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildApproveTx(HOLDER, SPENDER, 1_000_000n, 10_000);
      expect(builtInvocation(tx)).toEqual({
        fn: "approve",
        args: [HOLDER, SPENDER, 1_000_000n, 10_000],
      });
    });

    it("transfer: builds and submits SEP-41 transfer to standard address", async () => {
      stubWritePath();
      const client = new RwaAssetClient(signerConfig);
      const res = await client.transfer(ISSUER, HOLDER, 5_000_000n);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildTransferTx(ISSUER, HOLDER, 5_000_000n);
      expect(builtInvocation(tx)).toEqual({
        fn: "transfer",
        args: [ISSUER, HOLDER, 5_000_000n],
      });
    });

    it("transfer: builds and submits SEP-41 transfer to MuxedAddress (SEP-23)", async () => {
      stubWritePath();
      const kp = Keypair.random();
      const muxed = new MuxedAccount(new Account(kp.publicKey(), "0"), "12345");
      const mAddr = muxed.accountId();

      const client = new RwaAssetClient(signerConfig);
      const res = await client.transfer(ISSUER, mAddr, 2_500_000n);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildTransferTx(ISSUER, mAddr, 2_500_000n);
      expect(builtInvocation(tx)).toEqual({
        fn: "transfer",
        args: [ISSUER, mAddr, 2_500_000n],
      });
    });

    it("transferFrom: builds and submits SEP-41 transfer_from drawing on allowance", async () => {
      stubWritePath();
      const client = new RwaAssetClient(signerConfig);
      const res = await client.transferFrom(ISSUER, HOLDER, SPENDER, 1_000_000n);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildTransferFromTx(SPENDER, HOLDER, ISSUER, 1_000_000n);
      expect(builtInvocation(tx)).toEqual({
        fn: "transfer_from",
        args: [SPENDER, HOLDER, ISSUER, 1_000_000n],
      });
    });

    it("burn: builds and submits SEP-41 burn from caller balance", async () => {
      stubWritePath();
      const client = new RwaAssetClient(signerConfig);
      const res = await client.burn(ISSUER, 500_000n);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildBurnTx(HOLDER, 500_000n);
      expect(builtInvocation(tx)).toEqual({
        fn: "burn",
        args: [HOLDER, 500_000n],
      });
    });

    it("burnFrom: builds and submits SEP-41 burn_from drawing on allowance", async () => {
      stubWritePath();
      const client = new RwaAssetClient(signerConfig);
      const res = await client.burnFrom(ISSUER, HOLDER, 250_000n);
      expect(res).toEqual({ hash: SUBMITTED_HASH, ledger: 42 });

      const tx = await client.buildBurnFromTx(SPENDER, HOLDER, 250_000n);
      expect(builtInvocation(tx)).toEqual({
        fn: "burn_from",
        args: [SPENDER, HOLDER, 250_000n],
      });
    });
  });

  describe("Generic SEP-41 runner against RwaAssetClient", () => {
    it("executes standard SEP-41 sequence through generic client reference", async () => {
      stubWritePath();
      stubSimulation(succeeds(u32(7)));

      // Generic SEP-41 consumer that only knows about the Sep41TokenClient interface
      async function runSep41Workflow(token: Sep41TokenClient) {
        const decimals = await token.decimals();
        expect(decimals).toBe(7);

        const approveTx = await token.approve(ISSUER, SPENDER, 100n, 1_000);
        expect(approveTx.hash).toBe(SUBMITTED_HASH);

        const transferTx = await token.transfer(ISSUER, SPENDER, 50n);
        expect(transferTx.hash).toBe(SUBMITTED_HASH);

        const burnTx = await token.burn(ISSUER, 10n);
        expect(burnTx.hash).toBe(SUBMITTED_HASH);
      }

      const client = new RwaAssetClient(signerConfig);
      await runSep41Workflow(client);
    });
  });
});
