import { SetStateAction, useCallback, useEffect, useState } from "react";
import { getActiveUseCase, postUseCaseParticipation } from "../api/backend";
import { UseCaseConfig } from "../types";

function UseCaseComponent() {
  const [option, setOption] = useState<string>();
  const [activeUseCase, setActiveUseCase] = useState<string>();
  const [error, setError] = useState("");
  const [progress, setProgress] = useState(0);
  const [isDeregister, setIsDeregister] = useState(false);
  const [isSignUp, setIsSignUp] = useState(false);

  const useCases = [
    {
      name: "Energy Web Green Proofs",
      did: "did:web:dive-greenproofs.energywebx.com",
      url: "http://localhost:8000",
    },
    {
      name: "Track & Trace (via Energy Web)",
      did: "did:web:dive-ett-proxy.energywebx.com",
      url: "http://localhost:8000",
    },
    {
      name: "Energy Web Flex",
      did: "did:web:dive-flex.energywebx.com",
      url: "http://localhost:8000",
    },
    {
      name: "Energy Web Green Proofs",
      did: "did:web:dive-ev-supplier-switch.energywebx.com",
      url: "http://localhost:8000",
    },
    {
      name: "Example",
      did: "did:web:example.com",
      url: "http://localhost:8000",
    },
  ];

  const handleChange = useCallback(
    (e: { target: { value: SetStateAction<string> } }) => {
      setOption(e.target.value);
    },
    []
  );

  const handleSubmitUseCaseSelection = useCallback(async () => {
    if (option === undefined) {
      return;
    }

    const selectedUseCase = useCases.filter((a) => a.did === option);

    if (selectedUseCase.length === 0) {
      console.error("Selected use case does not exists");
      return;
    }

    const useCase = selectedUseCase[0];

    const { did, url } = useCase;

    setProgress(0);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);
    setIsSignUp(true);

    const config: UseCaseConfig = {
      notifyUseCase: true,
      updateServiceEndpoint: true,
      useCaseDidUrl: did,
      useCaseUrl: url,
    };

    await postUseCaseParticipation(config);

    setIsSignUp(false);
    clearInterval(interval);
  }, [option]);

  const handleSubmitUseCaseSelectionInvalidValue = useCallback(async () => {
    setProgress(0);
    setIsSignUp(true);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    const invalidConfig: UseCaseConfig = {
      notifyUseCase: true,
      updateServiceEndpoint: false,
      useCaseDidUrl: "invalid",
      useCaseUrl: "http://localhost:8000",
    };

    await postUseCaseParticipation(invalidConfig);

    setIsSignUp(false);
    clearInterval(interval);
  }, [option]);

  const handleSubmitUseCaseDeregistration = useCallback(async () => {
    setProgress(0);
    setIsDeregister(true);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    const config: UseCaseConfig = {
      notifyUseCase: false,
      updateServiceEndpoint: true,
      useCaseDidUrl: "deregistration",
      useCaseUrl: "",
    };

    await postUseCaseParticipation(config);

    clearInterval(interval);
    setIsDeregister(false);
  }, [option]);

  useEffect(() => {
    getActiveUseCase()
      .then((useCaseDidUrl) => {
        const useCase = useCases.find((a) => a.did === useCaseDidUrl);
        const activeUseCase = useCase ? useCase.name : "None";
        setActiveUseCase(activeUseCase);
      })
      .catch((e) => setError(error + "\n" + e.toString()));
  }, []);

  const isServerBlocked = isDeregister || isSignUp;

  return (
    <section className="box">
      <h3>Use Case</h3>
      {error !== "" && error}
      <fieldset>
        <legend>Aktueller Use Case</legend>
        <p>Die Anlage ist aktuell angemeldet für: {activeUseCase}</p>
        {/* invalidates the DIVE conflict token, e.g. empty string */}
        <button
          disabled={isServerBlocked}
          type="submit"
          onClick={handleSubmitUseCaseDeregistration}
        >
          Abmelden
        </button>

        {isDeregister && <progress max={60} value={progress} />}
      </fieldset>
      <fieldset>
        <legend>Wechsel oder erstmalige Anmeldung an einem Use Case</legend>
        <p>Auswahl aus der Liste der bekannten Use Cases</p>
        <p>
          <label>
            Use Case Name:
            <select name="Use Case" onChange={handleChange}>
              <option value="did:web:dive-greenproofs.energywebx.com">
                Energy Web Green Proofs
              </option>
              <option value="did:web:dive-ett-proxy.energywebx.com">
                Track & Trace (via Energy Web)
              </option>
              <option value="did:web:dive-flex.energywebx.com">
                Energy Web Flex
              </option>
              <option value="did:web:dive-ev-supplier-switch.energywebx.com">
                ReBeam: Lieferantenwechsel für BEV
              </option>
              <option value="did:web:example.com">Example</option>
            </select>
          </label>
        </p>
        <p>
          Bei der regulären Anmeldung wird der 'Konflikt-Token' vor der
          Anmeldung aktualisiert. Dies entspricht einer Abmeldung beim
          vorherigen Use Case und vermeidet daher mehrere, glechzeitige Use Case
          Teilnahmen.
        </p>
        {/* sets the DIVE conflict token to the correct value to participate in the did:web:use-case-name that the user selected from the dropdown
            then calls the use case api to register the device for participation */}
        <button
          type="submit"
          disabled={isServerBlocked}
          onClick={handleSubmitUseCaseSelection}
          title="Bei der regulären Anmeldung wird der 'Konflikt-Token' vor der Anmeldung aktualisiert. Dies entspricht einer Abmeldung beim vorherigen Use Case und vermeidet daher mehrere, glechzeitige Use Case Teilnahmen."
        >
          Anmelden (Regulär mit Abmeldung){" "}
        </button>
        {isSignUp && <progress max={60} value={progress} />}
        <p>
          Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der
          Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird
          fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case
          angemeldet.
        </p>
        {/* calls the user case api to register the device for participation without updating the DIVE conflict token (for demo purpose; will lead to an error) */}
        <button
          type="submit"
          disabled={isServerBlocked}
          onClick={handleSubmitUseCaseSelectionInvalidValue}
          title="Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case angemeldet."
        >
          Anmelden (ohne Abmeldung)
        </button>
        {isSignUp && <progress max={60} value={progress} />}
      </fieldset>
      <fieldset>
        <legend>Bekanntmachen</legend>
        <p>
          Fügt einen Use Case zur Liste der bekannten Use Cases hinzu. Danach
          kann die Anlage beim Use Case angemeldet werden.
        </p>
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
}

export default UseCaseComponent;
