/**
 * Write-path failures a caller has to tell apart before retrying.
 *
 * What matters to automation is whether a retry can execute the operation a
 * second time. A plain `Error` from the write path means the outcome is known:
 * the transaction was never accepted, or it was applied and failed. These two
 * cover a transaction the network accepted but that was not seen to settle.
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
