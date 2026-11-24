import { GovernanceClient } from "../src/index.js";
import { configFor, printResult, required, signerAddress } from "./common.js";

const support = process.env.VOTE_SUPPORT;
if (support !== "true" && support !== "false") throw new Error("Set VOTE_SUPPORT to true or false.");
const client = new GovernanceClient(configFor("governance"));
const result = await client.vote(
  signerAddress(),
  BigInt(required("PROPOSAL_ID")),
  support === "true",
  BigInt(process.env.VOTE_WEIGHT ?? "1"),
);
printResult(result);
