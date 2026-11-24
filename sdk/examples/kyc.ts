import { ComplianceClient } from "../src/index.js";
import { configFor, printResult, required, signerAddress } from "./common.js";

const level = Number(required("KYC_LEVEL"));
if (![0, 1, 2, 3].includes(level)) throw new Error("KYC_LEVEL must be 0, 1, 2, or 3.");
const client = new ComplianceClient(configFor("compliance"));
const result = await client.setKyc(signerAddress(), required("KYC_SUBJECT_ADDRESS"), {
  jurisdiction: process.env.KYC_JURISDICTION ?? "US",
  level: level as 0 | 1 | 2 | 3,
  expiresAt: Number(process.env.KYC_EXPIRES_AT ?? 0),
});
printResult(result);
