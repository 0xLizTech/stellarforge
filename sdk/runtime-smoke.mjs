import { nativeToScVal } from "@stellar/stellar-sdk";
import { RwaAssetClient, TESTNET_CONFIG } from "./dist/index.mjs";

// Test the package's real public read method with an in-memory testnet RPC
// response. This checks runtime module loading and BigInt/XDR decoding without
// depending on a deployed contract or making writes to the public network.
const client = new RwaAssetClient({
  ...TESTNET_CONFIG,
  contracts: { rwaAsset: "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM" },
});
client.server = {
  async simulateTransaction() {
    return { result: { retval: nativeToScVal(42n, { type: "i128" }) } };
  },
};

const supply = await client.totalSupply();
if (supply !== 42n) throw new Error(`Expected mocked testnet supply 42n; received ${supply}`);
console.log(`SDK read smoke passed: totalSupply() => ${supply}`);
