import type { DidUri, HexString, KiltAddress } from '@kiltprotocol/types';

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

