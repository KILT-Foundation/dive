import { Claim, Credential } from "@kiltprotocol/core";
import { FormEvent, useCallback, useEffect, useState } from "react";
import type { IClaimContents, DidUri } from "@kiltprotocol/types";

import {
  productionCtype,
  presentationCtype,
} from "../../ctypes";
import { getClaim, getCredential, postClaim } from "../../api/backend";

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
}: {
  boxDid: DidUri;
  boxDidPending: boolean;
  handleCreateBoxDIDClick: () => Promise<void>;
  ownerDidPending: boolean;
  progress: number;
  mode: Mode;
}) {
  // states
  const [claim, setClaim] = useState(undefined);
  const [credential, setCredential] = useState([]);

  const [error, setError] = useState("");

  // side effects

  useEffect(() => {
    getClaim()
      .then((claim) => setClaim(claim))
      .catch((e) => setError(error + "\n" + e.toString()));

    getCredential()
      .then((credentials) => setCredential(credentials))
      .catch((e) => setError(error + "\n" + e.toString()));
  }, [mode]);

  // Callbacks

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

      const unapprovedClaim = await postClaim(newCredential);
      setClaim(unapprovedClaim.contents);
    },
    [boxDid, mode]
  );

  return (
    <section>
      {error !== "" && error}

      <>
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
      </>
    </section>
  );
}

export default BoxComponent;
