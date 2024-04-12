import { Claim, Credential } from "@kiltprotocol/core";
import { FormEvent, useCallback, useEffect, useRef, useState } from "react";
import type { IClaimContents, DidUri } from "@kiltprotocol/types";

import {
  productionCtype,
  presentationCtype,
  selfIssuedCtype,
} from "../../ctypes";
import { getClaim, getCredential, postClaim } from "../../api/backend";
import {
  InjectedWindowProvider,
  getExtensions,
} from "@kiltprotocol/kilt-extension-api";
import { getSession } from "../../api/session";
import { fetchCredential } from "../../api/credential";

import {
  PresentationClaimSection,
  PresentationCredentialSection,
  ProductionClaimSection,
  ProductionCredentialSection,
} from "./CredentialClaim";
import { Mode } from "../../App";

function BoxComponent({
  boxDid,
  boxDidPending,
  handleCreateBoxDIDClick,
  ownerDidPending,
  progress,
  mode,
  setMode,
}: {
  boxDid: DidUri;
  boxDidPending: boolean;
  handleCreateBoxDIDClick: () => Promise<void>;
  ownerDidPending: boolean;
  progress: number;
  mode: Mode;
  setMode: (mode: Mode) => void;
}) {
  // states
  const [claim, setClaim] = useState(undefined);
  const [credential, setCredential] = useState([]);

  const [error, setError] = useState("");

  // side effects

  useEffect(() => {
    getClaim(mode)
      .then((claim) => setClaim(claim))
      .catch((e) => setError(error + "\n" + e.toString()));

    getCredential()
      .then((credentials) => setCredential(credentials))
      .catch((e) => setError(error + "\n" + e.toString()));
  }, [mode]);

  // Callbacks

  const handleModeSwitch = () => {
    if (mode === Mode.production) {
      setMode(Mode.presentation);
    } else {
      setMode(Mode.production);
    }
  };

  const handleClaimSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);

      const claimContent = Object.fromEntries(
        formData.entries()
      ) as IClaimContents;

      const ctype =
        mode === Mode.production ? productionCtype : presentationCtype;

      Object.entries(ctype.properties).forEach(([key, value]) => {
        if ("type" in value && value.type === "number" && key in claimContent) {
          claimContent[key] = parseInt(claimContent[key] as string, 10);
        }
      });

      const claim = Claim.fromCTypeAndClaimContents(
        ctype,
        claimContent,
        boxDid
      );

      const newCredential = Credential.fromClaim(claim);

      const unapprovedClaim = await postClaim(newCredential, mode);
      setClaim(unapprovedClaim.contents);
    },
    [boxDid, mode]
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
        <button style={{ margin: "10px" }} onClick={handleModeSwitch}>
          {mode}
        </button>

        {claim &&
          (mode === Mode.presentation ? (
            <PresentationCredentialSection
              credentials={credential}
              claim={claim}
            />
          ) : (
            <ProductionCredentialSection
              credentials={credential}
              claim={claim}
            />
          ))}
        {!claim &&
          (mode === Mode.presentation ? (
            <form onSubmit={handleClaimSubmit}>
              <PresentationClaimSection hasDid={!boxDid} />
            </form>
          ) : (
            <form onSubmit={handleClaimSubmit}>
              <ProductionClaimSection hasDid={!boxDid} />
            </form>
          ))}

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
