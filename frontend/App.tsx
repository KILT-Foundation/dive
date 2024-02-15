import { type FormEvent, useCallback, useState } from "react";
import ky from "ky";
import type { DidUri } from "@kiltprotocol/types";

import { useAsyncValue } from "./util/useAsyncValue";
import { getExistingDid, getPaymentAddress, API_URL } from "./api/backend";
import Footer from "./ui_components/FooterSection";
import OperatorComponent from "./ui_components/OperatorSection";
import BoxComponent from "./ui_components/BoxSection";

export function App() {
  // states
  const [boxDidPending, setBoxDidPending] = useState(false);
  const [progress, setProgress] = useState(0);
  const address = useAsyncValue(getPaymentAddress, []);
  const boxDid = useAsyncValue(getExistingDid, []);
  const [ownerDidPending, setOwnerDidPending] = useState(false);
  const [ownerDIDReady, setOwnerDIDReady] = useState(false);
  const [ownerDIDs, setOwnerDIDs] = useState<
    Array<{ did: DidUri; name?: string }>
  >([]);

  // Callbacks

  const handleCreateBoxDIDClick = useCallback(async () => {
    setProgress(0);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    try {
      setBoxDidPending(true);

      await ky.post(`${API_URL}/did`, { timeout: false }).json();
    } catch (error) {
      console.error(error);
    } finally {
      setBoxDidPending(false);
      clearInterval(interval);
      window.location.reload();
    }
  }, []);

  const handleCreateOwnerDIDClick = useCallback(
    async (event: FormEvent<HTMLButtonElement>) => {
      let interval: ReturnType<typeof setInterval>;

      try {
        setOwnerDidPending(true);

        if (!address) {
          throw new Error("Impossible: no address");
        }

        const { name } = event.currentTarget;
        const { getSignedDidCreationExtrinsic } = window.kilt[name];
        const { signedExtrinsic } = await getSignedDidCreationExtrinsic(
          address
        );

        setProgress(0);
        interval = setInterval(() => {
          setProgress((old) => old + 1);
        }, 1000);

        await ky.post(`${API_URL}/payment`, {
          json: signedExtrinsic,
          timeout: false,
        });
        confirm("Did is created!");

        setOwnerDIDReady(true);
      } catch (error) {
        console.error(error);
      } finally {
        setOwnerDidPending(false);
        clearInterval(interval);
      }
    },
    [address]
  );

  const handleGetOwnerDIDsClick = useCallback(
    async (event: FormEvent<HTMLButtonElement>) => {
      try {
        const { name } = event.currentTarget;
        const { getDidList } = window.kilt[name];
        setOwnerDIDs(await getDidList());
      } catch (error) {
        console.error(error);
      }
    },
    [address]
  );

  return (
    <>
      <BoxComponent
        boxDid={boxDid}
        boxDidPending={boxDidPending}
        handleCreateBoxDIDClick={handleCreateBoxDIDClick}
        ownerDidPending={ownerDidPending}
        progress={progress}
      />
      <OperatorComponent
        address={address}
        ownerDidPending={ownerDidPending}
        boxDidPending={boxDidPending}
        progress={progress}
        ownerDIDReady={ownerDIDReady}
        ownerDIDs={ownerDIDs}
        handleCreateOwnerDIDClick={handleCreateOwnerDIDClick}
        handleGetOwnerDIDsClick={handleGetOwnerDIDsClick}
      />
      <Footer />
    </>
  );
}
