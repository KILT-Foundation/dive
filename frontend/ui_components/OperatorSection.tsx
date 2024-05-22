import { FormEvent, Fragment, useCallback, useEffect, useState } from "react";
import type { DidUri, IClaimContents, KiltAddress } from "@kiltprotocol/types";
import {
  getExtensions,
  type InjectedWindowProvider,
} from "@kiltprotocol/kilt-extension-api";
import { Claim } from "@kiltprotocol/core";
import { selfIssuedCtype } from "../ctypes";
import { getSession } from "../api/session";
import { fetchCredential } from "../api/credential";

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

  const handleSelfCredential = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);
      const claimContent = Object.fromEntries(formData.entries());

      const extensions = getExtensions();
      const extensionName = "Sporran";
      const extension: InjectedWindowProvider = extensions.find(
        (val) => val.name === extensionName
      );

      const dids = await extension.getDidList();

      const owner = dids[0].did;

      const claim = Claim.fromCTypeAndClaimContents(
        selfIssuedCtype,
        claimContent as IClaimContents,
        owner
      );

      let session = await getSession(extension);
      await fetchCredential(session, claim);
    },
    []
  );

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
    <>
      {address && (
        <Fragment>
          {Object.entries(extensions).length === 0 && (
            <p>
              ❌️ KILT Wallet nicht vorhanden, bitte installieren{" "}
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

          <form onSubmit={handleSelfCredential}>
            <fieldset>
              <legend>Selbstauskunftszertifikat</legend>
              <p>
                <label>
                  Name: <input name="name" required />
                </label>
              </p>
              <p>
                <label>
                  Adresse: <input name="address" required />
                </label>
              </p>
              <button type="submit">Anfordern</button>
            </fieldset>
          </form>
        </Fragment>
      )}
    </>
  );
};

export default OperatorComponent;
