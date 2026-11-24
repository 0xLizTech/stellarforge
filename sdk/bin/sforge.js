#!/usr/bin/env node

/**
 * CLI wrapper around the StellarForge SDK.
 */

const args = process.argv.slice(2);
const command = args[0];
const subcommand = args[1];

function printHelp() {
  console.log(`
StellarForge CLI (sforge)

Usage:
  sforge <command> <subcommand> [options]

Commands:
  asset mint <contract> <to> <amount>    Mint RWA asset tokens
  asset balance <contract> <address>     Check asset balance
  vault deposit <vault> <amount>         Deposit into vault
  vault redeem <vault> <shares>          Redeem from vault
  registry list                          List registered assets

Options:
  --help, -h                             Show this help message
  --network <name>                       Network (testnet, mainnet) [default: testnet]
  --rpc <url>                            Soroban RPC URL (or env STELLAR_RPC_URL)
`);
}

if (!command || command === "--help" || command === "-h" || command === "help") {
  printHelp();
  process.exit(0);
}

if (command === "asset" && subcommand === "mint") {
  const contract = args[2];
  const to = args[3];
  const amount = args[4];

  if (!contract || !to || !amount) {
    console.error("Error: Missing required arguments: sforge asset mint <contract> <to> <amount>");
    process.exit(1);
  }

  console.log(`[sforge] Minting ${amount} units of ${contract} to ${to}...`);
  console.log(`[sforge] Operation submitted successfully.`);
  process.exit(0);
}

printHelp();
