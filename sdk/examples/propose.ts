import { GovernanceClient } from "../src/index.js";
import { configFor, printResult, required, signerAddress } from "./common.js";

const client = new GovernanceClient(configFor("governance"));
const result = await client.propose(
  signerAddress(),
  process.env.PROPOSAL_TITLE ?? "Example governance proposal",
  process.env.PROPOSAL_DESCRIPTION_SHA256 ?? "0".repeat(64),
  Number(process.env.PROPOSAL_VOTING_PERIOD_LEDGERS ?? 100_000),
);
console.log(`Proposal ${result.proposalId}`);
printResult(result);
