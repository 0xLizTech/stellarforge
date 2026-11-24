# Recipe: DAO Vote on an Asset Parameter (#211)

## Overview
This recipe demonstrates using the \`GovernanceClient\` to propose and vote on changing an asset parameter (e.g. transfer fee or compliance rule).

## Step-by-Step
1. **Create Proposal**:
   \`\`\`typescript
   const tx = await govClient.buildProposeTx(proposer, title, description, executionPayload);
   await govClient.submit(tx, proposer);
   \`\`\`
2. **Cast Vote**:
   \`\`\`typescript
   const voteTx = await govClient.buildVoteTx(voter, proposalId, true);
   await govClient.submit(voteTx, voter);
   \`\`\`
3. **Execute Approved Proposal**:
   \`\`\`typescript
   await govClient.executeProposal(admin, proposalId);
   \`\`\`
