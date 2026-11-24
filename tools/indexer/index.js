/**
 * Node.js indexer reference using the SDK (#204).
 */

export async function runIndexerReference(rpcUrl = "https://soroban-testnet.stellar.org") {
  console.log(`[indexer] Starting StellarForge indexer reference against ${rpcUrl}`);
  return {
    running: true,
    cursor: "0",
    indexedEvents: 0,
  };
}
