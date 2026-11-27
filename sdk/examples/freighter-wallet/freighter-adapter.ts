import {
  FreighterSigner,
  RwaAssetClient,
  TESTNET_CONFIG,
  toStroops,
  type NetworkConfig,
  type TxResult,
} from "@stellarforge-protocol/sdk";

/**
 * Minimal subset of `@stellar/freighter-api` (v2+) used by this example.
 *
 * In a bundled app, import these functions directly from
 * `@stellar/freighter-api` instead of declaring this interface.
 */
export interface FreighterApi {
  isConnected(): Promise<{ isConnected: boolean; error?: string }>;
  requestAccess(): Promise<{ address: string; error?: string }>;
  getNetworkDetails(): Promise<{
    network: string;
    networkPassphrase: string;
    error?: string;
  }>;
  signTransaction(
    xdr: string,
    opts?: { address?: string; networkPassphrase?: string },
  ): Promise<{ signedTxXdr: string; signerAddress: string; error?: string }>;
}

/**
 * Connects to Freighter, verifies that the wallet's active network passphrase
 * matches `network.networkPassphrase`, and returns an `RwaAssetClient` wired
 * with a `FreighterSigner`.
 */
export async function createFreighterRwaClient(
  freighter: FreighterApi,
  rwaAssetContractId: string,
  network: NetworkConfig = TESTNET_CONFIG,
): Promise<{ client: RwaAssetClient; address: string }> {
  const { isConnected, error: connectError } = await freighter.isConnected();
  if (connectError || !isConnected) {
    throw new Error(connectError ?? "Freighter browser extension is not installed or not connected.");
  }

  const { address, error: accessError } = await freighter.requestAccess();
  if (accessError || !address) {
    throw new Error(accessError ?? "Freighter did not return a public key.");
  }

  // Always verify Freighter's active network passphrase against the SDK config
  // before signing. A mismatch causes Freighter to sign against the wrong
  // network hash and Soroban RPC to reject the transaction with tx_bad_auth.
  const details = await freighter.getNetworkDetails();
  if (details.error) {
    throw new Error(`Unable to read Freighter network details: ${details.error}`);
  }
  if (details.networkPassphrase !== network.networkPassphrase) {
    throw new Error(
      `Network passphrase mismatch: Freighter is on "${details.networkPassphrase}", ` +
        `but the dApp expects "${network.networkPassphrase}". Switch networks in Freighter and retry.`,
    );
  }

  const signer = new FreighterSigner(address, (xdr, options) =>
    freighter.signTransaction(xdr, options),
  );

  const client = new RwaAssetClient({
    ...network,
    contracts: { rwaAsset: rwaAssetContractId },
    signer,
  });

  return { client, address };
}

/**
 * Transfers RWA tokens using the connected Freighter wallet.
 */
export async function transferWithFreighter(
  freighter: FreighterApi,
  rwaAssetContractId: string,
  recipient: string,
  humanAmount: string,
  network: NetworkConfig = TESTNET_CONFIG,
): Promise<TxResult> {
  const { client, address } = await createFreighterRwaClient(
    freighter,
    rwaAssetContractId,
    network,
  );
  const metadata = await client.metadata();
  const amount = toStroops(humanAmount, metadata.decimals);
  return client.transfer(address, recipient, amount);
}
