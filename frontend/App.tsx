import { type FormEvent, Fragment, useCallback, useEffect, useRef, useState } from 'react';
import ReactJson from 'react-json-view';
import ky from 'ky';
import { type DidUri, type KiltAddress, type ICredential, Credential, IClaim, IClaimContents, Claim, ICType, CType } from '@kiltprotocol/core';
import { useAsyncValue } from './useAsyncValue';
import oliLogo from './OLI.png';
import kiltLogo from './built-on-kilt.svg';

const ctype = {
  "$id": "kilt:ctype:0xc945ec1d1bc96dcef2c6f1198047c2be7edf7beb5f82c418a19c6614033c6256",
  "$schema": "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
  "additionalProperties": false,
  "properties": {
    "Art der Anlage": {
      "type": "string"
    },
    "Nennleistung (kW)": {
      "type": "number"
    },
    "Standort": {
      "type": "string"
    }
  },
  "title": "DIVE Anlagenzertifikat",
  "type": "object"
} as ICType
const apiUrl = '/api/v1';

async function getPaymentAddress() {
  const { address } = await ky.get(`${apiUrl}/payment`).json<{ address: KiltAddress }>();
  return address;
}

async function getExistingDid() {
  try {

    const { did } = await ky.get(`${apiUrl}/did`).json<{ did: DidUri }>();
    return did;
  } catch (exception) {
    console.error(exception);
    return undefined;
  }
}

interface Claim {
  'Art der Anlage': string;
  'Nennleistung (kW)': string;
  Standort: string;
}

async function getClaim() {
  try {
    const res = await ky.get(`${apiUrl}/claim`).json();
    const requestedCredential = JSON.parse(res.base_claim);
    return requestedCredential.claim.contents;[]
  } catch (exception) {
    console.error(exception);
    return undefined;
  }
}

async function getCredential() {
  try {
    let response = await ky.get(`${apiUrl}/credential`).json<ICredential>();

    let data = response[0];
    console.log(data, data.approved, data.credential)
    if (!data.approved) {
      return undefined
    }

    return data.credential
  } catch (exception) {
    console.error(exception);
    return undefined;
  }
}

export function App() {
  const [boxDid, setBoxDid] = useState<DidUri>();
  const [boxDidPending, setBoxDidPending] = useState(false);
  const [claim, setClaim] = useState<Claim>();
  const [credential, setCredential] = useState<ICredential>();
  const [progress, setProgress] = useState(0);

  const address = useAsyncValue(getPaymentAddress, []);
  const [ownerDidPending, setOwnerDidPending] = useState(false);
  const [ownerDIDReady, setOwnerDIDReady] = useState(false);
  const [ownerDIDs, setOwnerDIDs] = useState<Array<{ did: DidUri; name?: string }>>([]);
  const [extensions, setExtensions] = useState(window.kilt);

  useEffect(() => {
    function initialize() {
      setExtensions({ ...window.kilt });
    }
    window.addEventListener('kilt-extension#initialized', initialize);
    window.dispatchEvent(new CustomEvent('kilt-dapp#initialized'));
    return () => {
      window.removeEventListener('kilt-extension#initialized', initialize);
    }
  }, []);

  useEffect(() => {
    (async () => {
      const did = await getExistingDid();
      setBoxDid(did);
    })();
  }, []);

  useEffect(() => {
    (async () => {
      const claim = await getClaim();
      console.log(claim);
      setClaim(claim);
    })();
  }, []);

  useEffect(() => {
    (async () => {
      const credential = await getCredential();
      setCredential(credential);
    })();
  }, []);


  const handleCreateBoxDIDClick = useCallback(async () => {
    setProgress(0);
    const interval = setInterval(() => {
      setProgress((old) => old + 1);
    }, 1000);

    try {
      setBoxDidPending(true);

      await ky.post(`${apiUrl}/did`, { timeout: false }).json();
    } catch (error) {
      console.error(error);
    } finally {
      setBoxDidPending(false);
      clearInterval(interval);
      window.location.reload()
    }
  }, []);

  const handleCreateOwnerDIDClick = useCallback(async (event: FormEvent<HTMLButtonElement>) => {
    let interval: ReturnType<typeof setInterval>;

    try {
      setOwnerDidPending(true);

      if (!address) {
        throw new Error('Impossible: no address');
      }

      const { name } = event.currentTarget;
      const { getSignedDidCreationExtrinsic } = window.kilt[name];
      const { signedExtrinsic } = await getSignedDidCreationExtrinsic(address);

      setProgress(0);
      interval = setInterval(() => {
        setProgress((old) => old + 1);
      }, 1000);

      await ky.post(`${apiUrl}/payment`, { body: signedExtrinsic, timeout: false });
      confirm('Did is created!')

      setOwnerDIDReady(true);
    } catch (error) {
      console.error(error);
    } finally {
      setOwnerDidPending(false);
      clearInterval(interval);
    }
  }, [address]);

  const handleGetOwnerDIDsClick = useCallback(async (event: FormEvent<HTMLButtonElement>) => {
    try {
      const { name } = event.currentTarget;
      const { getDidList } = window.kilt[name];
      setOwnerDIDs(await getDidList());
    } catch (error) {
      console.error(error);
    }
  }, [address]);

  const handleClaimSubmit = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const json = Object.fromEntries(formData.entries());


    const newJson = {
      'Art der Anlage': json['Art der Anlage'],
      'Nennleistung (kW)': parseInt(json['Nennleistung (kW)'] as string, 10),
      Standort: json["Standort"],
    }

    console.log(newJson, boxDid)

    const newClaim = Claim.fromCTypeAndClaimContents(ctype, newJson, boxDid)
    const newCredential = Credential.fromClaim(newClaim);

    let claim = await ky.post(`${apiUrl}/claim`, { json: newCredential });
    setClaim(claim);
  }, [boxDid]);

  const credentialDialogRef = useRef<HTMLDialogElement>();
  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, [])

  const handleResetClick = useCallback(() => {
    if (!confirm('STOPP! Wirklich zurücksetzen?')) {
      return;
    }
    (async () => {
      await ky.delete('/api/v1/did');
      window.location.reload()
    })();
  }, [])

  return <section>
    <h1>OLI Box</h1>

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
          <p>✅️ Art der Anlage: {claim['Art der Anlage']}</p>
          <p>✅️ Nennleistung (kW): {claim['Nennleistung (kW)']}</p>
          <p>✅️ Standort: {claim.Standort}</p>
          {credential && (
            <p>
              ✅️ Zertifikat beglaubigt
              <button type="button" onClick={handleShowCredentialClick} id="credential">🔍️</button>
            </p>
          )}
          <dialog ref={credentialDialogRef}>
            <a href="https://polkadot.js.org/apps/#/chainstate" target="_blank" rel="noreferrer">Polkadot</a>
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
            <p><label>Art der Anlage: <input name="Art der Anlage" required /></label></p>
            <p><label>Nennleistung (kW): <input name="Nennleistung (kW)" required /></label></p>
            <p><label>Standort: <input name="Standort" required /></label></p>
            <button type="submit">Anfordern</button>
          </fieldset>
        </form>
      )}
    </section>

    <section className="box">
      <h3>Betreiber</h3>
      {address && (<Fragment>
        {Object.entries(extensions).length === 0 && (
          <p>
            ❌️ KILT Wallet nicht vorhanden, bitte installieren {' '}
            <a href="https://www.sporran.org/" target="_blank" rel="noreferrer">Sporran</a>!
          </p>
        )}

        {!ownerDidPending && <p>
          {Object.entries(extensions).map(([key, { name, getSignedDidCreationExtrinsic }]) => getSignedDidCreationExtrinsic && (
            <button
              type="button"
              key={key}
              name={key}
              onClick={handleCreateOwnerDIDClick}
              disabled={boxDidPending}
            >
              Identität erstellen mit {name}
            </button>
          ))}
        </p>}

        {ownerDidPending && <p><progress max={40} value={progress} /></p>}

        {ownerDIDReady && <p>
          {Object.entries(extensions).map(([key, { name, getDidList }]) => getDidList && (
            <button
              type="button"
              key={key}
              name={key}
              onClick={handleGetOwnerDIDsClick}
              disabled={boxDidPending}
            >
              Identität abfragen von {name}
            </button>
          ))}
        </p>}

        {ownerDIDs.length > 0 &&
          <ul>
            {ownerDIDs.map(({ did, name }) => <li key={did}>{did} {name && `(${name})`}</li>)}
          </ul>
        }
      </Fragment>)}
    </section>

    <img src={oliLogo} alt="OLI logo" width={116} height={76} className="oli" />
    <img src={kiltLogo} alt="Built on KILT" width={142} height={28} className="kilt" />

    <button type="reset" onClick={handleResetClick}>Zurücksetzen</button>
  </section>;
}
