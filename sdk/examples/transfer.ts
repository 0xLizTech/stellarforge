import { RwaAssetClient, toStroops } from "../src/index.js";
import { configFor, printResult, required, signerAddress } from "./common.js";

const client = new RwaAssetClient(configFor("rwaAsset"));
const result = await client.transfer(
  signerAddress(),
  required("TRANSFER_TO_ADDRESS"),
  toStroops(required("TRANSFER_AMOUNT")),
);
printResult(result);
