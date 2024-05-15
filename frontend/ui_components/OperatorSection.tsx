import { FormEvent, Fragment, useEffect, useState } from "react";
import type { DidUri, KiltAddress } from "@kiltprotocol/types";

const OperatorComponent = ({
  address,
  ownerDidPending,
  boxDidPending,
  progress,
  ownerDIDReady,
  ownerDIDs,
  handleCreateOwnerDIDClick,
  handleGetOwnerDIDsClick,
}: {
  address: KiltAddress;
  ownerDidPending: boolean;
  boxDidPending: boolean;
  progress: number;
  ownerDIDReady: boolean;
  ownerDIDs: { did: DidUri; name?: string }[];
  handleCreateOwnerDIDClick: (
    event: FormEvent<HTMLButtonElement>
  ) => Promise<void>;
  handleGetOwnerDIDsClick: (
    event: FormEvent<HTMLButtonElement>
  ) => Promise<void>;
}) => {
  const [extensions, setExtensions] = useState(window.kilt);

  // useEffects
  useEffect(() => {
    function initialize() {
      setExtensions({ ...window.kilt });
    }
    window.addEventListener("kilt-extension#initialized", initialize);
    window.dispatchEvent(new CustomEvent("kilt-dapp#initialized"));
    return () => {
      window.removeEventListener("kilt-extension#initialized", initialize);
    };
  }, []);

  return (
    <section className="box">
      <h3>Betreiber</h3>
      {address && (
        <Fragment>
          {Object.entries(extensions).length === 0 && (
            <p>
              ❌️ KILT Wallet nicht vorhanden, bitte installieren{' '}
              <a
                href="https://www.sporran.org/"
                target="_blank"
                rel="noreferrer"
              >
                Sporran
              </a>
              !
            </p>
          )}

          {!ownerDidPending && (
            <p>
              {Object.entries(extensions).map(
                ([key, { name, getSignedDidCreationExtrinsic }]) =>
                  getSignedDidCreationExtrinsic && (
                    <button
                      type="button"
                      key={key}
                      name={key}
                      onClick={handleCreateOwnerDIDClick}
                      disabled={boxDidPending}
                    >
                      Identität erstellen mit {name}
                    </button>
                  )
              )}
            </p>
          )}

          {ownerDidPending && (
            <p>
              <progress max={40} value={progress} />
            </p>
          )}

          {ownerDIDReady && (
            <p>
              {Object.entries(extensions).map(
                ([key, { name, getDidList }]) =>
                  getDidList && (
                    <button
                      type="button"
                      key={key}
                      name={key}
                      onClick={handleGetOwnerDIDsClick}
                      disabled={boxDidPending}
                    >
                      Identität abfragen von {name}
                    </button>
                  )
              )}
            </p>
          )}

          {ownerDIDs.length > 0 && (
            <ul>
              {ownerDIDs.map(({ did, name }) => (
                <li key={did}>
                  {did} {name && `(${name})`}
                </li>
              ))}
            </ul>
          )}
        </Fragment>
      )}
    </section>
  );
};

export default OperatorComponent;
