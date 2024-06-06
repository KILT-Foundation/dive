import { Fragment, useCallback, useEffect, useRef, useState } from "react";
import ReactJson from "react-json-view";
import { cType } from "../../ctypes";
import { AttestationResponse } from "../../types";

const entries = [
  "Art der Anlage",
  "Nennleistung (kW)",
  "Standort",
  "SMGW ID",
  "Meter ID",
  "Messlokations-ID",
  "Marktlokations-ID",
];

export function ClaimSection({ hasDid }: { hasDid: boolean }) {
  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      <p>
        Art der Anlage: <input name="Art der Anlage" required />
      </p>
      <p>
        <label>
          Nennleistung (kW):
          <input name="Nennleistung (kW)" required type="number" step="any" />
        </label>
      </p>
      <p>
        <label>
          Standort: <input name="Standort" required />
        </label>
      </p>
      <p>
        <label>
          SMGW ID: <input name="SMGW ID" required />
        </label>
      </p>
      <p>
        <label>
          Meter ID: <input name="Meter ID" required />
        </label>
      </p>
      <p>
        <label>
          Messlokations-ID: <input name="Messlokations-ID" required />
        </label>
      </p>
      <p>
        <label>
          Marktlokations-ID: <input name="Marktlokations-ID" required />
        </label>
      </p>

      <button disabled={hasDid} type="submit">
        Anfordern
      </button>
    </fieldset>
  );
}

export function CredentialSection({ credentials, claim }) {
  const credentialDialogRef = useRef<HTMLDialogElement>();
  const [credential, setCredential] = useState<AttestationResponse>(undefined);

  useEffect(() => {
    const targetCredential = credentials.find(
      (credential) =>
        `kilt:ctype:${credential.credential.claim.cTypeHash}` ===
          cType.$id && credential.approved
    );
    setCredential(targetCredential);
  }, [credentials]);

  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, []);

  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      {entries.map((key) => (
        <p key={key}>
          {key in claim && "✅️ "}
          {key}: {claim[key]}
        </p>
      ))}
      {credential && credential.approved && !credential.revoked && (
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
      {credential && credential.approved && credential.revoked && (
        <p>❌ Zertifikat Widerruft</p>
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
        <ReactJson src={credential ? credential.credential : []} />
      </dialog>

      {credential && (
        <Fragment>
          <p>Status: {credential.revoked ? "Widerrufen" : "Beglaubigt"}</p>
          <p>Credential hash: {credential.credential.rootHash}</p>
          <p>CType: {credential.credential.claim.cTypeHash}</p>
        </Fragment>
      )}

      {!credential && <p>💡️ Zertifikat in Bearbeitung</p>}
    </fieldset>
  );
}
