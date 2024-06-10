import { Claim, Credential } from "@kiltprotocol/core";
import { FormEvent, useCallback, useEffect, useState } from "react";
import type { IClaimContents, DidUri } from "@kiltprotocol/types";

import { cType } from "../../ctypes";
import { getClaim, getCredential, postClaim } from "../../api/backend";

import { ClaimSection, CredentialSection } from "./CredentialClaim";

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
  }, []);

  // Callbacks

  const handleClaimSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const formData = new FormData(event.currentTarget);

      const claimContent = Object.fromEntries(
        formData.entries()
      ) as IClaimContents;

      Object.entries(cType.properties).forEach(([key, value]) => {
        if ("type" in value && value.type === "number" && key in claimContent) {
          claimContent[key] = parseFloat(claimContent[key] as string);
        }

        if (claimContent[key] === "" || Number.isNaN(claimContent[key])) {
          delete claimContent[key];
        }
      });

      try {
        const claim = Claim.fromCTypeAndClaimContents(
          cType,
          claimContent,
          boxDid
        );

        const newCredential = Credential.fromClaim(claim);

        const unapprovedClaim = await postClaim(newCredential);
        setClaim(unapprovedClaim.contents);
      } catch (e) {
        console.error(e.cause);
        setError(e.toString());
        return;
      }
    },
    [boxDid]
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

        {claim && <CredentialSection credentials={credential} claim={claim} />}
        {!claim && (
          <form onSubmit={handleClaimSubmit}>
            <ClaimSection hasDid={!boxDid} />
          </form>
        )}
      </>
    </section>
  );
}

export default BoxComponent;
