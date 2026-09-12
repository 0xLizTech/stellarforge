// ─── Core Protocol Types ──────────────────────────────────────────────────────

export type AssetClass =
  | "real_estate"
  | "commodity"
  | "private_equity"
  | "debt"
  | "infrastructure"
  | "art"
  | "other";

export interface AssetMetadata {
  /** Human-readable name of the real-world asset */
  name: string;
  /** Ticker symbol, e.g. "REIT-NYC-001" */
  symbol: string;
  /** Number of decimal places (Stellar standard: 7) */
  decimals: number;
  /** Broad category of the underlying asset */
  assetClass: AssetClass;
  /** IPFS CID or SHA-256 hex of the legal offering document */
  legalDocHash: string;
  /** Hard cap on total minted supply; 0 means uncapped */
  maxSupply: bigint;
}

export interface KycRecord {
  /** ISO-3166-1 alpha-2 country code */
  jurisdiction: string;
  /** 0=none | 1=basic | 2=full | 3=accredited */
  level: 0 | 1 | 2 | 3;
  /** Unix timestamp of expiry, or 0 for never */
  expiresAt: number;
}

export interface NetworkConfig {
  /** "mainnet" | "testnet" | "futurenet" | custom RPC URL */
  network: "mainnet" | "testnet" | "futurenet" | (string & {});
  /** Soroban RPC endpoint URL */
  rpcUrl: string;
  /** Stellar network passphrase */
  networkPassphrase: string;
}

export interface ContractAddresses {
  rwaAsset?: string;
  registry?: string;
  compliance?: string;
  governance?: string;
}

export interface StellarForgeConfig extends NetworkConfig {
  contracts: ContractAddresses;
  /** Optional signing keypair (StrKey secret, `S...`) — only for server-side use */
  signerSecret?: string;
}

/** Outcome of a write transaction that was submitted and confirmed. */
export interface TxResult {
  /** Hex-encoded transaction hash. */
  hash: string;
  /** Ledger sequence the transaction was included in. */
  ledger: number;
}

// ─── Well-known network presets ───────────────────────────────────────────────

export const TESTNET_CONFIG: Omit<NetworkConfig, "network"> & { network: "testnet" } = {
  network: "testnet",
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
};

export const MAINNET_CONFIG: Omit<NetworkConfig, "network"> & { network: "mainnet" } = {
  network: "mainnet",
  rpcUrl: "https://soroban-rpc.stellar.org",
  networkPassphrase: "Public Global Stellar Network ; September 2015",
};
