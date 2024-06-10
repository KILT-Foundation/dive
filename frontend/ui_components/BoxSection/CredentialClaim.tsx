import { Fragment, useCallback, useEffect, useRef, useState } from "react";
import ReactJson from "react-json-view";
import { cType } from "../../ctypes";
import { AttestationResponse } from "../../types";

const entries = [
  "Art der Anlage",
  "Anschlussnetzbetreiber",
  "Betreiber",
  "Betreiberstatus",
  "Bruttoleistung",
  "EEG Inbetriebnahmedatum",
  "EEG Registrierungsdatum",
  "Errichtungsort(Lage)",
  "Inbetriebnahmedatum",
  "Installierte Leistung",
  "Marktlokations-ID",
  "Messlokations-ID",
  "Meter ID",
  "Name der Einheit",
  "Registrierungsdatum im aktuellen Betriebsstatus",
  "SMGW ID",
  "Standort",
  "Wechselrichterleistung",
];

export function ClaimSection({ hasDid }: { hasDid: boolean }) {
  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      <p>
        Art der Anlage:
        <label>
          <input
            name="Art der Anlage"
            value="Solar"
            type="radio"
            defaultChecked
          />
          Solar
        </label>
        <label>
          <input name="Art der Anlage" value="Speicher" type="radio" />
          Speicher
        </label>
      </p>
      <p>
        <label>
          Betreiber: <input name="Betreiber" />
        </label>
      </p>
      <p>
        <label>
          Betreiberstatus: <input name="Betreiberstatus" />
        </label>
      </p>
      <p>
        <label>
          Standort: <input name="Standort" />
        </label>
      </p>
      <p>
        <label>
          Errichtungsort (Lage): <input name="Errichtungsort (Lage)" />
        </label>
      </p>
      <p>
        <label>
          Name der Einheit: <input name="Name der Einheit" />
        </label>
      </p>
      <p>
        <label>
          Bruttoleistung:
          <input name="Bruttoleistung" type="number" step="any" />
        </label>
      </p>
      <p>
        <label>
          Wechselrichterleistung:
          <input name="Wechselrichterleistung" type="number" step="any" />
        </label>
      </p>
      <p>
        <label>
          Inbetriebnahmedatum: <input name="Inbetriebnahmedatum" type="date" />
        </label>
      </p>
      <p>
        <label>
          Anschlussnetzbetreiber: <input name="Anschlussnetzbetreiber" />
        </label>
      </p>
      <p>
        <label>
          Registrierungsdatum im aktuellen Betriebsstatus:
          <input
            name="Registrierungsdatum im aktuellen Betriebsstatus"
            type="date"
          />
        </label>
      </p>
      <p>
        <label>
          Installierte Leistung:
          <input name="Installierte Leistung" type="number" step="any" />
        </label>
      </p>
      <p>
        <label>
          EEG Inbetriebnahmedatum:
          <input name="EEG Inbetriebnahmedatum" type="date" />
        </label>
      </p>
      <p>
        <label>
          EEG Registrierungsdatum:
          <input name="EEG Registrierungsdatum" type="date" />
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
        `kilt:ctype:${credential.credential.claim.cTypeHash}` === cType.$id &&
        credential.approved
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
        <p key={key}>{key in claim && `${key}: ${claim[key]} ✅️`}</p>
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
