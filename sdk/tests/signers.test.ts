import { Account, Asset, Keypair, Networks, Operation, Transaction, TransactionBuilder } from "@stellar/stellar-sdk";
import { describe, expect, it, vi } from "vitest";

import { FreighterSigner, KeypairSigner } from "../src/signers.js";

const KEYPAIR = Keypair.fromRawEd25519Seed(Buffer.alloc(32, 7));

function unsignedTransaction(): Transaction {
  return new TransactionBuilder(new Account(KEYPAIR.publicKey(), "1"), {
    fee: "100",
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(Operation.payment({ destination: KEYPAIR.publicKey(), asset: Asset.native(), amount: "1" }))
    .setTimeout(30)
    .build();
}

describe("transaction signers", () => {
  it("signs prepared transactions with a keypair", async () => {
    const signer = KeypairSigner.fromSecret(KEYPAIR.secret());
    const signed = await signer.signTransaction(unsignedTransaction());

    expect(signer.publicKey).toBe(KEYPAIR.publicKey());
    expect(signed.signatures).toHaveLength(1);
  });

  it("adapts a Freighter-style XDR signer", async () => {
    const signTransaction = vi.fn(async (transactionXdr: string, options: { networkPassphrase: string }) => {
      const transaction = TransactionBuilder.fromXDR(transactionXdr, options.networkPassphrase) as Transaction;
      transaction.sign(KEYPAIR);
      return { signedTxXdr: transaction.toXDR(), signerAddress: KEYPAIR.publicKey() };
    });
    const signer = new FreighterSigner(KEYPAIR.publicKey(), signTransaction);
    const signed = await signer.signTransaction(unsignedTransaction(), Networks.TESTNET);

    expect(signTransaction).toHaveBeenCalledWith(expect.any(String), {
      address: KEYPAIR.publicKey(),
      networkPassphrase: Networks.TESTNET,
    });
    expect(signed.signatures).toHaveLength(1);
  });

  it("rejects a Freighter response signed by a different account", async () => {
    const signer = new FreighterSigner(KEYPAIR.publicKey(), async (transactionXdr, options) => ({
      signedTxXdr: transactionXdr,
      signerAddress: Keypair.fromRawEd25519Seed(Buffer.alloc(32, 8)).publicKey(),
    }));

    await expect(signer.signTransaction(unsignedTransaction(), Networks.TESTNET)).rejects.toThrow(
      /expected/,
    );
  });
});