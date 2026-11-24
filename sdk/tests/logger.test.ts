import { describe, it, expect, vi } from "vitest";
import { RwaAssetClient } from "../src/client.js";
import { SilentLogger } from "../src/logger.js";
import type { Logger } from "../src/logger.js";
import type { StellarForgeConfig } from "../src/types.js";

describe("Logger", () => {
  const baseConfig: StellarForgeConfig = {
    network: "testnet",
    networkPassphrase: "Test SDF Network ; September 2015",
    rpcUrl: "https://soroban-testnet.stellar.org",
    contracts: {
      rwaAsset: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
      compliance: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
      registry: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
      governance: "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
    },
  };

  it("defaults to SilentLogger", () => {
    const client = new RwaAssetClient(baseConfig);
    expect(client.logger).toBeInstanceOf(SilentLogger);
  });

  it("accepts a custom logger", () => {
    const customLogger: Logger = {
      debug: vi.fn(),
      info: vi.fn(),
      warn: vi.fn(),
      error: vi.fn(),
    };

    const client = new RwaAssetClient({
      ...baseConfig,
      logger: customLogger,
    });

    expect(client.logger).toBe(customLogger);
    client.logger.debug("test message");
    expect(customLogger.debug).toHaveBeenCalledWith("test message");
  });
});
