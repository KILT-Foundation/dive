import { FormEvent, Fragment, useEffect, useState } from "react";
import type { DidUri, KiltAddress } from "@kiltprotocol/types";

const UseCaseComponent = ({
}: {
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
      <h3>Use Case</h3>
      <fieldset>
            <legend>Aktueller Use Case</legend>
            <p>Die Anlage ist aktuell angemeldet für: Energy Web Green Proofs</p>
            {/* invalidates the DIVE conflict token, e.g. empty string */}
            <button type="submit">Abmelden</button>
        </fieldset>
        <fieldset>
            <legend>Wechsel oder erstmalige Anmeldung an einem Use Case</legend>
            <p>Auswahl aus der Liste der bekannten Use Cases</p>
            <p>
                <label>
                    Use Case Name:
                    <select name="Use Case">
                        <option value="did:web:dive-greenproofs.energywebx.com">Energy Web Green Proofs</option>
                        <option value="did:web:dive-ett-proxy.energywebx.com">Track & Trace (via Energy Web)</option>
                        <option value="did:web:dive-flex.energywebx.com">Energy Web Flex</option>
                        <option value="did:web:dive-ev-supplier-switch.energywebx.com">ReBeam: Lieferantenwechsel für BEV</option>
                    </select>
                </label>
            </p>
            <p>
                Bei der regulären Anmeldung wird der 'Konflikt-Token' vor der Anmeldung aktualisiert. Dies entspricht einer Abmeldung beim vorherigen Use Case und vermeidet daher mehrere, glechzeitige Use Case Teilnahmen.
            </p>
            {/* sets the DIVE conflict token to the correct value to participate in the did:web:use-case-name that the user selected from the dropdown
            then calls the use case api to register the device for participation */}
            <button type="submit" title="Bei der regulären Anmeldung wird der 'Konflikt-Token' vor der Anmeldung aktualisiert. Dies entspricht einer Abmeldung beim vorherigen Use Case und vermeidet daher mehrere, glechzeitige Use Case Teilnahmen.">Anmelden (Regulär mit Abmeldung)</button>
            <p>
                Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case angemeldet.
            </p>
            {/* calls the user case api to register the device for participation without updating the DIVE conflict token (for demo purpose; will lead to an error) */}
            <button type="submit" title="Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case angemeldet.">Anmelden (ohne Abmeldung)</button>
        </fieldset>
      <fieldset>
            <legend>Bekanntmachen</legend>
            <p>Fügt einen Use Case zur Liste der bekannten Use Cases hinzu. Danach kann die Anlage beim Use Case angemeldet werden.</p>
            <p>
                <label>
                    Use Case DID: <input name="New Use Case DID" required />
                </label>
            </p>
            {/* Adds an Use Case by did:web url to the device list, retrieves Use Case friendly name, API endpoint and public key from did doc and stores it in device storage */}
            <button type="submit">Hinzufügen</button>
        </fieldset>
    </section>
  );
};

export default UseCaseComponent;
