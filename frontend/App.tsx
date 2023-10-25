import { type FormEvent, Fragment, useCallback, useEffect, useRef, useState } from 'react';
import ReactJson from 'react-json-view';
import ky from 'ky';
import { type DidUri, type KiltAddress, type ICredential } from '@kiltprotocol/sdk-js';
import { useAsyncValue } from './useAsyncValue';
import oliLogo from './OLI.png';
import kiltLogo from './built-on-kilt.svg';

const apiUrl = '/api/v1';

async function getPaymentAddress() {
  // return '4tTFsj531ZFqyhdYnWmzKU3gWGN68qYPBSKkB7UJ5XZWCAyg' as KiltAddress;
  const { address } = await ky.get(`${apiUrl}/payment`).json<{ address: KiltAddress }>();
  return address;
}

async function getExistingDid() {
  try {
    // return 'did:kilt:4rrkiRTZgsgxjJDFkLsivqqKTqdUTuxKk3FX3mKFAeMxsR5E';
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
    // return {
    //   'Art der Anlage': 'OLI',
    //   'Nennleistung (kW)': '120',
    //   Standort: 'Musterstraße 1, 12345 Musterstadt',
    // };
    return await ky.get(`${apiUrl}/claim`).json<Claim>();
  } catch (exception) {
    console.error(exception);
    return undefined;
  }
}

async function getCredential() {
  try {
    // return {
    //   'claim': {
    //     'cTypeHash': '0xad52bd7a8bd8a52e03181a99d2743e00d0a5e96fdc0182626655fcf0c0a776d0',
    //     'contents': { 'Username': 'arty-name', 'User ID': '133055' },
    //     'owner': 'did:kilt:4rrkiRTZgsgxjJDFkLsivqqKTqdUTuxKk3FX3mKFAeMxsR5E',
    //   },
    //   'legitimations': [],
    //   'claimHashes': ['0x73ab53e3e87960ae33b827d8bde3fee2717cfd5af2841d7dfc163a0eeed85474', '0xbd0d90cff6b3784e9e53afb0499076902c677c992c472b9f4aac87fe0f700709', '0xfacb2590ec33b9c5c1cd37bc5da8023629052d1fd593f4b9fb5c3271e7bee146'],
    //   'claimNonceMap': {
    //     '0x39df1673e48bcdf17a1eff936fbe2460555de5bdc029b515afd25bb81012ebcd': '56ea4c72-caa8-425a-9def-fa5ea5571fcc',
    //     '0xc9cccabfbfc0c529263c97d9775ed8297df7832d53948229c7282667c2d15f7c': 'd6faf781-9a0c-4f10-a58d-591f35f3f6ad',
    //     '0x800e8346b87610819d18304201c9aaee24ef2f69769e86713928937e37ffff99': '4a4d173c-c348-4c24-b974-9c6e84817a92',
    //   },
    //   'rootHash': '0x202b70def75caa7d2130524b12d759e711ebf75960e838cbbc27d657560e6675',
    //   'delegationId': null,
    // } as ICredential;
    return await ky.get(`${apiUrl}/credential`).json<ICredential>();
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
    const json = Object.fromEntries(formData.entries()) as unknown as Claim;
    await ky.post(`${apiUrl}/claim`, { json });
    setClaim(json);
  }, []);

  const credentialDialogRef = useRef<HTMLDialogElement>();
  const handleShowCredentialClick = useCallback(() => {
    credentialDialogRef.current?.showModal();
  }, [])

  const handleResetClick = useCallback(() => {
    if (!confirm('STOPP! Wirklich zurücksetzen?')) {
      return;
    }
    (async () => {
      await ky.delete('/did');
      alert('Was haben wir getan? 😱️');
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
          {boxDidPending && <progress max={40} value={progress} />}
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
