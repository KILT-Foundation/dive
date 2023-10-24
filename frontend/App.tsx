import { type FormEvent, Fragment, useCallback, useEffect, useState } from 'react';
import ky from 'ky';
import { type DidUri, type KiltAddress } from '@kiltprotocol/sdk-js';
import { useAsyncValue } from './useAsyncValue';
import oliLogo from './OLI.png';
import kiltLogo from './built-on-kilt.svg';

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
  Betreiber: string;
  Adresse: string;
}

async function getClaim() {
  try {
    // return { Betreiber: 'OLI', Adresse: 'Musterstraße 1, 12345 Musterstadt' };
    return await ky.get(`${apiUrl}/claim`).json<Claim>();
  } catch (exception) {
    console.error(exception);
    return undefined;
  }
}

export function App() {
  const [boxDid, setBoxDid] = useState<DidUri>();
  const [boxDidPending, setBoxDidPending] = useState(false);
  const [claim, setClaim] = useState<Claim>();
  const [progress, setProgress] = useState(0);

  const address = useAsyncValue(getPaymentAddress, []);
  const [ownerDidPending, setOwnerDidPending] = useState(false);
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
    } catch (error) {
      console.error(error);
    } finally {
      setOwnerDidPending(false);
      clearInterval(interval);
    }
  }, [address]);

  const handleClaimSubmit = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const json = Object.fromEntries(formData.entries()) as unknown as Claim;
    await ky.post(`${apiUrl}/claim`, { json });
    setClaim(json);
  }, []);

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
        <Fragment>
          <p>✅️ Betreiber: {claim.Betreiber}</p>
          <p>✅️ Adresse: {claim.Adresse}</p>
        </Fragment>
      )}
      {!claim && (
        <form onSubmit={handleClaimSubmit}>
          <fieldset>
            <legend>Stammdatenzertifikat</legend>
            <p><label>Betreiber: <input name="Betreiber" required /></label></p>
            <p><label>Adresse: <input name="Adresse" required /></label></p>
            <button type="submit">Anfordern</button>
          </fieldset>
        </form>
      )}
    </section>

    <section className="box">
      <h3>Betreiber</h3>
      {address && (
        <p>
          {Object.entries(extensions).length === 0 && (
            <span>
              ❌️ KILT Wallet nicht vorhanden, bitte installieren {' '}
              <a href="https://www.sporran.org/" target="_blank" rel="noreferrer">Sporran</a>!
            </span>
          )}

          {!ownerDidPending && <Fragment>
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
          </Fragment>}

          {ownerDidPending && <progress max={40} value={progress} />}
        </p>
      )}
    </section>

    <img src={oliLogo} alt="OLI logo" width={116} height={76} className="oli" />
    <img src={kiltLogo} alt="Built on KILT" width={142} height={28} className="kilt" />

    <button type="reset" onClick={handleResetClick}>Zurücksetzen</button>
  </section>;
}
