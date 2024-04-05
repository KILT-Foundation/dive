import { Claim, Credential } from "@kiltprotocol/core";
import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import ReactJson from "react-json-view";
import type { IClaimContents, DidUri } from "@kiltprotocol/types";

import { certificateCtype, selfIssuedCtype } from "../ctypes";
import { getClaim, getCredential, postClaim } from "../api/backend";
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
  const [claim, setClaim] = useState(undefined);
  const [credential, setCredential] = useState(undefined);
  const [error, setError] = useState("");

  // callbacks

  useEffect(() => {
    getClaim()
      .then((claim) => setClaim(claim))
      .catch((e) => setError(error + "\n" + e.toString()));

    getCredential()
      .then((credential) => setCredential(credential))
      .catch((e) => setError(error + "\n" + e.toString()));
  }, []);

  // Callbacks
  const credentialDialogRef = useRef<HTMLDialogElement>();
  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, []);

  const handleClaimSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);
      const claimContent = Object.fromEntries(formData.entries()) as IClaimContents;

      Object.entries(certificateCtype.properties).forEach(([key, value]) => {
        if ('type' in value && value.type === "number" && key in claimContent) {
          claimContent[key] = parseInt(claimContent[key] as string, 10);
        }
      });

      const claim = Claim.fromCTypeAndClaimContents(
        certificateCtype,
        claimContent,
        boxDid
      );
      const newCredential = Credential.fromClaim(claim);

      const unapprovedClaim = await postClaim(newCredential);
      setClaim(unapprovedClaim.contents);
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
      {error !== "" && error}

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
            {[
              'Vorname',
              'Nachname',
              'Geburtsdatum',
              'Straße und Hausnummer',
              'Postleitzahl',
              'Standort',
              'Telefonnummer',
              'E-Mail',
              'Steuernummer',
              'Umsatzsteueridentifikationsnummer',
              'Zeitpunkt der Installation',
              'Arbeitsvermögensbegrenzung',
              'Bezeichnung (Anbieterintern)',
              'BNetzA-Kraftwerksnummer',
              'CO2-Äquivalent-Emission',
              'EEG-Anlagenschlüssel',
              'EIC-W der Einheit',
              'Marktlokations-ID',
              'Marktstammdatenregister-ID',
              'Maximal-Leistung',
              'Minimal-Leistung',
              'Messlokations-ID',
              'Meter ID',
              'Nennleistung (W)',
              'Obere Grenze der Leistungsregelung (W)',
              'untere Grenze der Leistungsregelung',
              'Primärenergieträger',
              'SMGW ID',
              'Spannungsebene',
              'Technologie',
              'Unternehmenstyp',
              'Verteilnetzbetreiber',
            ].map((key) => (
              <p key={key}>
                {(key in claim) && '✅️'}
                {key}: {claim[key]}
              </p>
            ))}
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
                  Vorname: <input name="Vorname" autoComplete="given-name"/>
                </label>
              </p>
              <p>
                <label>
                  Nachname: <input name="Nachname" autoComplete="family-name"/>
                </label>
              </p>
              <p>
                <label>
                  Geburtsdatum: <input name="Geburtsdatum" type="date"/>
                </label>
              </p>
              <p>
                <label>
                  Straße und Hausnummer: <input name="Straße und Hausnummer" autoComplete="address-line1"/>
                </label>
              </p>
              <p>
                <label>
                  Postleitzahl: <input name="Postleitzahl" autoComplete="postal-code"/>
                </label>
              </p>
              <p>
                <label>
                  Standort: <input name="Standort" autoComplete="address-level2"/>
                </label>
              </p>
              <p>
                <label>
                  Telefonnummer: <input name="Telefonnummer" type="tel" autoComplete="tel"/>
                </label>
              </p>
              <p>
                <label>
                  E-Mail: <input name="E-Mail" type="email"/>
                </label>
              </p>
              <p>
                <label>
                  Steuernummer: <input name="Steuernummer"/>
                </label>
              </p>
              <p>
                <label>
                  Umsatzsteueridentifikationsnummer: <input name="Umsatzsteueridentifikationsnummer"/>
                </label>
              </p>
              <p>
                <label>
                  Zeitpunkt der Installation: <input name="Zeitpunkt der Installation" type="date"/>
                </label>
              </p>
              <p>
                <label>
                  Arbeitsvermögensbegrenzung: <input name="Arbeitsvermögensbegrenzung"/>
                </label>
              </p>
              <p>
                <label>
                  Bezeichnung (Anbieterintern): <input name="Bezeichnung (Anbieterintern)"/>
                </label>
              </p>
              <p>
                <label>
                  BNetzA-Kraftwerksnummer: <input name="BNetzA-Kraftwerksnummer"/>
                </label>
              </p>
              <p>
                <label>
                  CO2-Äquivalent-Emission: <input name="CO2-Äquivalent-Emission"/>
                </label>
              </p>
              <p>
                <label>
                  EEG-Anlagenschlüssel: <input name="EEG-Anlagenschlüssel"/>
                </label>
              </p>
              <p>
                <label>
                  EIC-W der Einheit: <input name="EIC-W der Einheit"/>
                </label>
              </p>
              <p>
                <label>
                  Marktlokations-ID: <input name="Marktlokations-ID"/>
                </label>
              </p>
              <p>
                <label>
                  Marktstammdatenregister-ID: <input name="Marktstammdatenregister-ID"/>
                </label>
              </p>
              <p>
                <label>
                  Maximal-Leistung: <input name="Maximal-Leistung" type="number"/>
                </label>
              </p>
              <p>
                <label>
                  Minimal-Leistung: <input name="Minimal-Leistung" type="number"/>
                </label>
              </p>
              <p>
                <label>
                  Messlokations-ID: <input name="Messlokations-ID"/>
                </label>
              </p>
              <p>
                <label>
                  Meter ID: <input name="Meter ID"/>
                </label>
              </p>
              <p>
                <label>
                  Nennleistung (W): <input name="Nennleistung (W)"/>
                </label>
              </p>
              <p>
                <label>
                  Obere Grenze der Leistungsregelung (W): <input name="Obere Grenze der Leistungsregelung (W)"
                                                                 type="number"/>
                </label>
              </p>
              <p>
                <label>
                  untere Grenze der Leistungsregelung: <input name="untere Grenze der Leistungsregelung" type="number"/>
                </label>
              </p>
              <p>
                <label>
                  Primärenergieträger: <input name="Primärenergieträger"/>
                </label>
              </p>
              <p>
                <label>
                  SMGW ID: <input name="SMGW ID"/>
                </label>
              </p>
              <p>
                <label>
                  Spannungsebene: <input name="Spannungsebene"/>
                </label>
              </p>
              <p>
                <label>
                  Technologie: <input name="Technologie"/>
                </label>
              </p>
              <p>
                <label>
                  Unternehmenstyp: <input name="Unternehmenstyp"/>
                </label>
              </p>
              <p>
                <label>
                  Verteilnetzbetreiber: <input name="Verteilnetzbetreiber"/>
                </label>
              </p>
              <button type="submit">Anfordern</button>
            </fieldset>
          </form>
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
      </section>
    </section>
  );
}

export default BoxComponent;
