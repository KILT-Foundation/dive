import { useCallback, useRef } from "react";
import ReactJson from "react-json-view";

const productionEntries = [
  "Vorname",
  "Nachname",
  "Geburtsdatum",
  "Straße und Hausnummer",
  "Postleitzahl",
  "Standort",
  "Telefonnummer",
  "E-Mail",
  "Steuernummer",
  "Umsatzsteueridentifikationsnummer",
  "Zeitpunkt der Installation",
  "Arbeitsvermögensbegrenzung",
  "Bezeichnung (Anbieterintern)",
  "BNetzA-Kraftwerksnummer",
  "CO2-Äquivalent-Emission",
  "EEG-Anlagenschlüssel",
  "EIC-W der Einheit",
  "Marktlokations-ID",
  "Marktstammdatenregister-ID",
  "Maximal-Leistung",
  "Minimal-Leistung",
  "Messlokations-ID",
  "Meter ID",
  "Nennleistung (W)",
  "Obere Grenze der Leistungsregelung (W)",
  "untere Grenze der Leistungsregelung",
  "Primärenergieträger",
  "SMGW ID",
  "Spannungsebene",
  "Technologie",
  "Unternehmenstyp",
  "Verteilnetzbetreiber",
];

const presentationEntries = ["Art der Anlage", "Nennleistung (kW)", "Standort"];

export function PresentationClaimSection({ hasDid }: { hasDid: boolean }) {
  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      <p>
        Art der Anlage: <input name="Art der Anlage" required />
      </p>
      <p>
        <label>
          Nennleistung (kW): Unternehmenstyp:
          <input name="Nennleistung (kW)" required type="number" />
        </label>
      </p>
      <p>
        <label>
          Standort: <input name="Standort" required />
        </label>
      </p>

      <button disabled={hasDid} type="submit">
        Anfordern
      </button>
    </fieldset>
  );
}

export function ProductionClaimSection({ hasDid }: { hasDid: boolean }) {
  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      <p>
        <label>
          Vorname: <input name="Vorname" autoComplete="given-name" />
        </label>
      </p>
      <p>
        <label>
          Nachname: <input name="Nachname" autoComplete="family-name" />
        </label>
      </p>
      <p>
        <label>
          Geburtsdatum: <input name="Geburtsdatum" type="date" />
        </label>
      </p>
      <p>
        <label>
          Straße und Hausnummer:{" "}
          <input name="Straße und Hausnummer" autoComplete="address-line1" />
        </label>
      </p>
      <p>
        <label>
          Postleitzahl: <input name="Postleitzahl" autoComplete="postal-code" />
        </label>
      </p>
      <p>
        <label>
          Standort: <input name="Standort" autoComplete="address-level2" />
        </label>
      </p>
      <p>
        <label>
          Telefonnummer:{" "}
          <input name="Telefonnummer" type="tel" autoComplete="tel" />
        </label>
      </p>
      <p>
        <label>
          E-Mail: <input name="E-Mail" type="email" />
        </label>
      </p>
      <p>
        <label>
          Steuernummer: <input name="Steuernummer" />
        </label>
      </p>
      <p>
        <label>
          Umsatzsteueridentifikationsnummer:{" "}
          <input name="Umsatzsteueridentifikationsnummer" />
        </label>
      </p>
      <p>
        <label>
          Zeitpunkt der Installation:{" "}
          <input name="Zeitpunkt der Installation" type="date" />
        </label>
      </p>
      <p>
        <label>
          Arbeitsvermögensbegrenzung:{" "}
          <input name="Arbeitsvermögensbegrenzung" />
        </label>
      </p>
      <p>
        <label>
          Bezeichnung (Anbieterintern):{" "}
          <input name="Bezeichnung (Anbieterintern)" />
        </label>
      </p>
      <p>
        <label>
          BNetzA-Kraftwerksnummer: <input name="BNetzA-Kraftwerksnummer" />
        </label>
      </p>
      <p>
        <label>
          CO2-Äquivalent-Emission: <input name="CO2-Äquivalent-Emission" />
        </label>
      </p>
      <p>
        <label>
          EEG-Anlagenschlüssel: <input name="EEG-Anlagenschlüssel" />
        </label>
      </p>
      <p>
        <label>
          EIC-W der Einheit: <input name="EIC-W der Einheit" />
        </label>
      </p>
      <p>
        <label>
          Marktlokations-ID: <input name="Marktlokations-ID" />
        </label>
      </p>
      <p>
        <label>
          Marktstammdatenregister-ID:{" "}
          <input name="Marktstammdatenregister-ID" />
        </label>
      </p>
      <p>
        <label>
          Maximal-Leistung: <input name="Maximal-Leistung" type="number" />
        </label>
      </p>
      <p>
        <label>
          Minimal-Leistung: <input name="Minimal-Leistung" type="number" />
        </label>
      </p>
      <p>
        <label>
          Messlokations-ID: <input name="Messlokations-ID" />
        </label>
      </p>
      <p>
        <label>
          Meter ID: <input name="Meter ID" />
        </label>
      </p>
      <p>
        <label>
          Nennleistung (W): <input name="Nennleistung (W)" />
        </label>
      </p>
      <p>
        <label>
          Obere Grenze der Leistungsregelung (W):{" "}
          <input name="Obere Grenze der Leistungsregelung (W)" type="number" />
        </label>
      </p>
      <p>
        <label>
          untere Grenze der Leistungsregelung:{" "}
          <input name="untere Grenze der Leistungsregelung" type="number" />
        </label>
      </p>
      <p>
        <label>
          Primärenergieträger: <input name="Primärenergieträger" />
        </label>
      </p>
      <p>
        <label>
          SMGW ID: <input name="SMGW ID" />
        </label>
      </p>
      <p>
        <label>
          Spannungsebene: <input name="Spannungsebene" />
        </label>
      </p>
      <p>
        <label>
          Technologie: <input name="Technologie" />
        </label>
      </p>
      <p>
        <label>
          Unternehmenstyp: <input name="Unternehmenstyp" />
        </label>
      </p>
      <p>
        <label>
          Verteilnetzbetreiber: <input name="Verteilnetzbetreiber" />
        </label>
      </p>
      <button disabled={hasDid} type="submit">
        Anfordern
      </button>
    </fieldset>
  );
}

export function PresentationCredentialSection({ credential, claim }) {
  return (
    <CredentialSection
      credential={credential}
      claim={claim}
      entries={presentationEntries}
    />
  );
}

export function ProductionCredentialSection({ credential, claim }) {
  return (
    <CredentialSection
      credential={credential}
      claim={claim}
      entries={productionEntries}
    />
  );
}

function CredentialSection({ credential, claim, entries }) {
  const credentialDialogRef = useRef<HTMLDialogElement>();

  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, []);
  return (
    <fieldset>
      <legend>DIVE Anlagenzertifikat</legend>
      {entries.map((key) => (
        <p key={key}>
          {key in claim && "✅️"}
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
  );
}
