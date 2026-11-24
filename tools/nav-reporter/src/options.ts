import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

export interface CliOptions {
  appraisalPath: string | undefined;
  oracleContractId: string | undefined;
  appraiserKey: string | undefined;
  reporterKeyFile: string | undefined;
  relayerKeyFile: string | undefined;
  networkName: "testnet" | "mainnet";
  rpcUrl: string | undefined;
  dryRun: boolean;
  jsonOutput: boolean;
}

type ConfigFile = Partial<Record<
  "appraisal" | "oracle" | "appraiserKey" | "reporterKeyFile" | "relayerKeyFile" |
  "network" | "rpcUrl" | "dryRun" | "json",
  string | boolean
>>;

const VALUE_FLAGS: Record<string, keyof ConfigFile> = {
  "--appraisal": "appraisal",
  "--oracle": "oracle",
  "--appraiser-key": "appraiserKey",
  "--reporter-key-file": "reporterKeyFile",
  "--relayer-key-file": "relayerKeyFile",
  "--network": "network",
  "--rpc-url": "rpcUrl",
};

function parseArgs(args: string[]): { values: Partial<ConfigFile>; booleans: Partial<ConfigFile>; configPath?: string } {
  const values: Partial<ConfigFile> = {};
  const booleans: Partial<ConfigFile> = {};
  let configPath: string | undefined;
  for (let i = 0; i < args.length; i++) {
    const flag = args[i]!;
    if (flag === "--config") {
      if (!args[i + 1] || args[i + 1]!.startsWith("--")) throw new Error("--config requires a path");
      configPath = args[++i];
    } else if (flag === "--dry-run" || flag === "--json") {
      booleans[flag === "--dry-run" ? "dryRun" : "json"] = true;
    } else if (flag === "--no-dry-run" || flag === "--no-json") {
      booleans[flag === "--no-dry-run" ? "dryRun" : "json"] = false;
    } else {
      const key = VALUE_FLAGS[flag];
      if (!key) throw new Error(`Unknown option: ${flag}`);
      if (!args[i + 1] || args[i + 1]!.startsWith("--")) throw new Error(`${flag} requires a value`);
      values[key] = args[++i];
    }
  }
  return { values, booleans, ...(configPath ? { configPath } : {}) };
}

function readConfig(filePath: string | undefined): ConfigFile {
  if (!filePath) return {};
  if (!existsSync(filePath)) throw new Error(`Config file not found: ${filePath}`);
  let parsed: unknown;
  try {
    parsed = JSON.parse(readFileSync(filePath, "utf8"));
  } catch (error) {
    throw new Error(`Invalid config JSON: ${error instanceof Error ? error.message : String(error)}`);
  }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Config file must contain a JSON object");
  }
  const config = parsed as ConfigFile;
  for (const key of ["appraisal", "oracle", "appraiserKey", "reporterKeyFile", "relayerKeyFile", "network", "rpcUrl"] as const) {
    if (config[key] !== undefined && typeof config[key] !== "string") {
      throw new Error(`Config field ${key} must be a string`);
    }
  }
  for (const key of ["dryRun", "json"] as const) {
    if (config[key] !== undefined && typeof config[key] !== "boolean") {
      throw new Error(`Config field ${key} must be a boolean`);
    }
  }
  return config;
}

/** CLI flags override env vars, which override sforge.config.json, then defaults. */
export function resolveCliOptions(
  args: string[],
  env: NodeJS.ProcessEnv = process.env,
  cwd = process.cwd(),
): CliOptions {
  const { values, booleans, configPath } = parseArgs(args);
  const defaultConfigPath = path.join(cwd, "sforge.config.json");
  const selectedConfigPath = configPath
    ? path.resolve(cwd, configPath)
    : existsSync(defaultConfigPath) ? defaultConfigPath : undefined;
  const config = readConfig(selectedConfigPath);
  const configDir = selectedConfigPath ? path.dirname(selectedConfigPath) : cwd;

  const pick = (flagKey: keyof ConfigFile, envKey: string): string | undefined => {
    const cliValue = values[flagKey];
    if (typeof cliValue === "string") return cliValue;
    const envValue = env[envKey];
    if (envValue !== undefined) return envValue;
    const configValue = config[flagKey];
    return typeof configValue === "string" ? configValue : undefined;
  };
  const pickPath = (flagKey: keyof ConfigFile, envKey: string): string | undefined => {
    const cliValue = values[flagKey];
    if (typeof cliValue === "string") return path.resolve(cwd, cliValue);
    const envValue = env[envKey];
    if (envValue !== undefined) return path.resolve(cwd, envValue);
    const configValue = config[flagKey];
    return typeof configValue === "string" ? path.resolve(configDir, configValue) : undefined;
  };
  const pickBoolean = (key: "dryRun" | "json", envKey: string): boolean => {
    if (typeof booleans[key] === "boolean") return booleans[key] as boolean;
    if (env[envKey] !== undefined) {
      const value = env[envKey]!.toLowerCase();
      if (["true", "1", "yes"].includes(value)) return true;
      if (["false", "0", "no"].includes(value)) return false;
      throw new Error(`${envKey} must be true or false`);
    }
    return typeof config[key] === "boolean" ? config[key] as boolean : false;
  };

  const network = pick("network", "NETWORK") ?? "testnet";
  if (network !== "testnet" && network !== "mainnet") throw new Error("NETWORK must be 'testnet' or 'mainnet'");
  return {
    appraisalPath: pickPath("appraisal", "APPRAISAL_FILE"),
    oracleContractId: pick("oracle", "ORACLE_CONTRACT_ID"),
    appraiserKey: pick("appraiserKey", "APPRAISER_PUBLIC_KEY"),
    reporterKeyFile: pickPath("reporterKeyFile", "REPORTER_KEY_FILE"),
    relayerKeyFile: pickPath("relayerKeyFile", "RELAYER_KEY_FILE"),
    networkName: network,
    rpcUrl: pick("rpcUrl", "SOROBAN_RPC_URL"),
    dryRun: pickBoolean("dryRun", "DRY_RUN"),
    jsonOutput: pickBoolean("json", "JSON_OUTPUT"),
  };
}
