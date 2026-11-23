import type { StellarForgeConfig } from "./types.js";

/**
 * Mock client for application unit tests (#202).
 */
export class MockStellarForgeClient {
  readonly config: StellarForgeConfig;

  constructor(config: StellarForgeConfig) {
    this.config = config;
  }

  async simulateTransaction(): Promise<{ success: boolean; hash: string }> {
    return { success: true, hash: "mock-tx-hash-00000000000000000000000000000000000000000000000000000000" };
  }

  async getBalance(_address: string): Promise<bigint> {
    return 10_000_000_000n;
  }
}
