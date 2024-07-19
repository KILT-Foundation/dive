import { SetStateAction, useCallback, useEffect, useState } from "react";
import { getActiveUseCase, postUseCaseParticipation } from "../api/backend";
import { UseCaseConfig } from "../types";

const RawUseCases = [
  {
    name: "Energy Web Green Proofs",
    did: "did:web:dive-greenproofs.energywebx.com",
    url: "https://greenproofs.dive.energyweb.org",
  },
  {
    name: "Track & Trace (via Energy Web)",
    did: "did:web:dive-ett-proxy.energywebx.com",
    url: "https://greenproofs.dive.energyweb.org",
  },
  {
    name: "Energy Web Flex",
    did: "did:web:dive-flex.energywebx.com",
    url: "https://greenproofs.dive.energyweb.org",
  },
  {
    name: "Energy Web Green Proofs",
    did: "did:web:dive-ev-supplier-switch.energywebx.com",
    url: "https://greenproofs.dive.energyweb.org",
  },
  {
    name: "Example",
    did: "did:gp-dive-dev.energyweb.org",
    url: "https://gp-dive-dev.energyweb.org/",
  },
];

function UseCaseComponent() {
  // states
  const [option, setOption] = useState<string>();
  const [activeUseCase, setActiveUseCase] = useState<string>();
  const [error, setError] = useState("");
  const [customUseCase, setCustomUseCase] = useState<string>("");
  const [progress, setProgress] = useState(0);
  const [isDeregister, setIsDeregister] = useState(false);
  const [isAccepted, setIsAccepted] = useState(false);
  const [isSignUpValid, setIsSignUpValid] = useState(false);
  const [isSignUpInvalid, setIsSignUpInvalid] = useState(false);
  const [useCases, setUseCases] = useState(RawUseCases);

  // helper functions

  const updateActiveUserCase = (useCaseDidUrl: string) => {
    const useCase = useCases.find((a) => a.did === useCaseDidUrl);
    const activeUseCase = useCase ? useCase.name : "None";
    setActiveUseCase(activeUseCase);
  };

  // side effects

  useEffect(() => {
    getActiveUseCase()
      .then((useCaseDidUrl) => {
        updateActiveUserCase(useCaseDidUrl);
      })
      .catch((e) => setError(error + "\n" + e.toString()));
  }, []);

  // callbacks

  const handleAddUseCase = useCallback(() => {
    const upd = [...useCases];
    upd.push({
      did: customUseCase,
      name: customUseCase,
      url: "http://greenproofs.dive.energyweb.org",
    });

    setUseCases(upd);
    setCustomUseCase("");
  }, [customUseCase]);

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
    setIsSignUpValid(true);

    const config: UseCaseConfig = {
      notifyUseCase: true,
      updateServiceEndpoint: true,
      useCaseDidUrl: did,
      useCaseUrl: url,
    };

    const activeUseCase = await postUseCaseParticipation(config);
    updateActiveUserCase(activeUseCase);

    setIsSignUpValid(false);
    clearInterval(interval);
  }, [option]);

  const handleApproveTermsAndConditions = useCallback(() => {
    setIsAccepted(!isAccepted);
  }, [isAccepted]);

  const handleSubmitUseCaseSelectionInvalidValue = useCallback(async () => {
    setProgress(0);
    setIsSignUpInvalid(true);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    const invalidConfig: UseCaseConfig = {
      notifyUseCase: true,
      updateServiceEndpoint: false,
      useCaseDidUrl: "invalid",
      useCaseUrl: "http://greenproofs.dive.energyweb.org",
    };

    const activeUseCase = await postUseCaseParticipation(invalidConfig);
    updateActiveUserCase(activeUseCase);

    setIsSignUpInvalid(false);
    clearInterval(interval);
  }, [option]);

  const handleSubmitUseCaseDeregistration = useCallback(async () => {
    setProgress(0);
    setIsDeregister(true);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    const deregisterConfig: UseCaseConfig = {
      notifyUseCase: false,
      updateServiceEndpoint: true,
      useCaseDidUrl: "deregistration",
      useCaseUrl: "",
    };

    const activeUseCase = await postUseCaseParticipation(deregisterConfig);
    updateActiveUserCase(activeUseCase);

    clearInterval(interval);
    setIsDeregister(false);
  }, [option]);

  const isServerBlocked = isDeregister || isSignUpInvalid || isSignUpValid;

  return (
    <>
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

        {isDeregister && (
          <progress max={90} style={{ marginLeft: "1em" }} value={progress} />
        )}
      </fieldset>
      <fieldset>
        <legend>Wechsel oder erstmalige Anmeldung an einem Use Case</legend>
        <p>Auswahl aus der Liste der bekannten Use Cases</p>
        <p>
          <label>
            Use Case Name:
            <select
              name="Use Case"
              onChange={handleChange}
              disabled={isServerBlocked}
            >
              {useCases.map((useCase) => (
                <option value={useCase.did}> {useCase.name} </option>
              ))}
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
          disabled={isServerBlocked || !isAccepted}
          onClick={handleSubmitUseCaseSelection}
          title="Bei der regulären Anmeldung wird der 'Konflikt-Token' vor der Anmeldung aktualisiert. Dies entspricht einer Abmeldung beim vorherigen Use Case und vermeidet daher mehrere, glechzeitige Use Case Teilnahmen."
        >
          Anmelden (Regulär mit Abmeldung){" "}
        </button>
        {isSignUpValid && (
          <progress max={90} style={{ marginLeft: "1em" }} value={progress} />
        )}
        <p>
          Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der
          Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird
          fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case
          angemeldet.
        </p>
        {/* calls the user case api to register the device for participation without updating the DIVE conflict token (for demo purpose; will lead to an error) */}
        <button
          type="submit"
          disabled={isServerBlocked || !isAccepted}
          onClick={handleSubmitUseCaseSelectionInvalidValue}
          title="Die Anmeldung ohne vorherige Abmeldung dient nur zur Demonstration der Funktionsweise der Konfliktvermeidung. Die Anmeldung am Use Case wird fehlschlagen. Die Anlage wird folglich nicht tatsächlich beim Use Case angemeldet."
        >
          Anmelden (ohne Abmeldung)
        </button>
        {isSignUpInvalid && (
          <progress max={90} style={{ marginLeft: "1em" }} value={progress} />
        )}
        <div
          style={{
            marginTop: "1rem",
            display: "flex",
            flexDirection: "row",
            fontSize: "0.8rem",
          }}
        >
          <input
            type="checkbox"
            id="acceptTermsAndConditions"
            onClick={handleApproveTermsAndConditions}
            checked={isAccepted}
          />
          <label htmlFor="acceptTermsAndConditions">
            Ich stimme den AGB und der Datenschutzerklärung zu{" "}
          </label>
        </div>
      </fieldset>
      <fieldset>
        <legend>Bekanntmachen</legend>
        <p>
          Fügt einen Use Case zur Liste der bekannten Use Cases hinzu. Danach
          kann die Anlage beim Use Case angemeldet werden.
        </p>
        <p>
          <label>
            Use Case DID:{" "}
            <input
              name="New Use Case DID"
              value={customUseCase}
              required
              onChange={(e) => setCustomUseCase(e.target.value)}
            />
          </label>
        </p>
        {/* Adds an Use Case by did:web url to the device list, retrieves Use Case friendly name, API endpoint and public key from did doc and stores it in device storage */}
        <button type="submit" onClick={handleAddUseCase}>
          Hinzufügen
        </button>
      </fieldset>
    </>
  );
}

export default UseCaseComponent;
