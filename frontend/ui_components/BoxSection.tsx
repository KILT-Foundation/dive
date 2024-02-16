import { Claim, Credential } from "@kiltprotocol/core";
import { FormEvent, useCallback, useRef } from "react";
import ReactJson from "react-json-view";
import ky from "ky";
import type { IClaimContents, DidUri } from "@kiltprotocol/types";

import { certificateCtype, selfIssuedCtype } from "../ctypes";
import { API_URL, getClaim, getCredential } from "../api/backend";
import { useAsyncValue } from "../util/useAsyncValue";
import {
  InjectedWindowProvider,
  getExtensions,
} from "@kiltprotocol/kilt-extension-api";
import { getSession } from "../api/session";
import { fetchCredential } from "../api/credential";

function BoxComponent({
  boxDid,
  boxDidPending,
  handleCreateBoxDIDClick,
  ownerDidPending,
  progress,
}: {
  boxDid: DidUri;
  boxDidPending: boolean;
  handleCreateBoxDIDClick: () => Promise<void>;
  ownerDidPending: boolean;
  progress: number;
}) {
  // async states
  const credential = useAsyncValue(getCredential, []);
  const claim = useAsyncValue(getClaim, []);

  // Callbacks
  const credentialDialogRef = useRef<HTMLDialogElement>();
  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, []);

  const handleClaimSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);
      const json = Object.fromEntries(formData.entries());

      const claimContent = {
        "Art der Anlage": json["Art der Anlage"],
        "Nennleistung (kW)": parseInt(json["Nennleistung (kW)"] as string, 10),
        Standort: json["Standort"],
      } as IClaimContents;

      const claim = Claim.fromCTypeAndClaimContents(
        certificateCtype,
        claimContent,
        boxDid
      );
      const newCredential = Credential.fromClaim(claim);

      await ky.post(`${API_URL}/claim`, { json: newCredential });
    },
    [boxDid]
  );

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

  return (
    <section>
      <h1>OLI Box</h1>

      <section className="box">
        <h3>Anlage</h3>
        {boxDid && <p>✅️ Identität: {boxDid}</p>}
        {!boxDid && (
          <p>
            Noch keine Identität vorhanden
            {!boxDidPending && (
              <button
                className="init"
                type="button"
                onClick={handleCreateBoxDIDClick}
                disabled={ownerDidPending}
              >
                Identität erstellen!
              </button>
            )}
            {boxDidPending && <progress max={60} value={progress} />}
          </p>
        )}

        {claim && (
          <fieldset>
            <legend>DIVE Anlagenzertifikat</legend>
            <p>✅️ Art der Anlage: {claim["Art der Anlage"]}</p>
            <p>✅️ Nennleistung (kW): {claim["Nennleistung (kW)"]}</p>
            <p>✅️ Standort: {claim["Standort"]}</p>
            {credential && (
              <p>
                ✅️ Zertifikat beglaubigt
                <button
                  type="button"
                  onClick={handleShowCredentialClick}
                  id="credential"
                >
                  🔍️
                </button>
              </p>
            )}
            <dialog ref={credentialDialogRef}>
              <a
                href="https://polkadot.js.org/apps/#/chainstate"
                target="_blank"
                rel="noreferrer"
              >
                Polkadot
              </a>
              <form method="dialog">
                <button type="submit">✖️</button>
              </form>
              <ReactJson src={credential} />
            </dialog>
            {!credential && <p>💡️ Zertifikat in Bearbeitung</p>}
          </fieldset>
        )}
        {!claim && (
          <form onSubmit={handleClaimSubmit}>
            <fieldset>
              <legend>DIVE Anlagenzertifikat</legend>
              <p>
                <label>
                  Art der Anlage: <input name="Art der Anlage" required />
                </label>
              </p>
              <p>
                <label>
                  Nennleistung (kW):{" "}
                  <input name="Nennleistung (kW)" required type="number" />
                </label>
              </p>
              <p>
                <label>
                  Standort: <input name="Standort" required />
                </label>
              </p>
              <button type="submit">Anfordern</button>
            </fieldset>
          </form>
        )}

        <form onSubmit={handleSelfCredential}>
          <fieldset>
            <legend>Selbstauskunfts Zertifikat</legend>
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
      </section>
    </section>
  );
}

export default BoxComponent;
