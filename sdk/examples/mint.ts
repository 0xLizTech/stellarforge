import { RwaAssetClient, toStroops } from "../src/index.js";
import { configFor, printResult, required, signerAddress } from "./common.js";

const client = new RwaAssetClient(configFor("rwaAsset"));
const result = await client.mint(
  signerAddress(),
  required("MINT_TO_ADDRESS"),
  toStroops(required("MINT_AMOUNT")),
);
printResult(result);
