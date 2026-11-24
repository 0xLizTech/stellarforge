import { Keypair, TransactionBuilder } from "@stellar/stellar-sdk";

import type { Transaction } from "@stellar/stellar-sdk";

import type { Signer } from "./types.js";

/** Signs transactions with a Stellar secret-key-backed keypair. */
export class KeypairSigner implements Signer {
  readonly publicKey: string;

  /** @param keypair - The keypair whose public key authorizes submitted writes. */
  constructor(private readonly keypair: Keypair) {
    this.publicKey = keypair.publicKey();
  }

  /** Creates a signer from a StrKey secret. Keep this server-side only. */
  static fromSecret(secret: string): KeypairSigner {
    return new KeypairSigner(Keypair.fromSecret(secret));
  }

  async signTransaction(transaction: Transaction, _networkPassphrase?: string): Promise<Transaction> {
    transaction.sign(this.keypair);
    return transaction;
  }
}

/** Freighter-compatible result for signing a transaction XDR. */
export interface FreighterSignResult {
  /** Base64 XDR of the transaction signed by the wallet. */
  signedTxXdr: string;
  /** Public key of the account that signed the transaction. */
  signerAddress: string;
}

/** Freighter-style transaction signing callback, without a Freighter package dependency. */
export type FreighterSignTransaction = (
  transactionXdr: string,
  options: { address: string; networkPassphrase: string },
) => Promise<FreighterSignResult>;

/**
 * Adapts a Freighter-compatible wallet callback to the SDK's {@link Signer}
 * interface.
 */
export class FreighterSigner implements Signer {
  constructor(
    readonly publicKey: string,
    private readonly freighterSignTransaction: FreighterSignTransaction,
  ) {}

  async signTransaction(
    transaction: Transaction,
    networkPassphrase: string,
  ): Promise<Transaction> {
    const result = await this.freighterSignTransaction(transaction.toXDR(), {
      address: this.publicKey,
      networkPassphrase,
    });
    if (result.signerAddress !== this.publicKey) {
      throw new Error(
        `Freighter signed with ${result.signerAddress}, expected ${this.publicKey}`,
      );
    }
    return TransactionBuilder.fromXDR(result.signedTxXdr, networkPassphrase) as Transaction;
  }
}