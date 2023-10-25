import { type DidUri, type HexString, type KiltAddress } from '@kiltprotocol/sdk-js';

export {};

declare global {
  interface Window {
    kilt: Record<
      string,
      {
        name?: string;
        getSignedDidCreationExtrinsic?: (submitter: KiltAddress) => Promise<{ signedExtrinsic: HexString }>
        getDidList?: () => Promise<Array<{ did: DidUri; name?: string }>>;
      }
    >;
  }
}

