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

export function App() {
  const [boxDid, setBoxDid] = useState<DidUri>();
  const [boxDidPending, setBoxDidPending] = useState(false);
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


  const handleCreateBoxDIDClick = useCallback(async () => {
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

  return <section>
    <h1>OLI-Box</h1>

    <section>
      <h3>Anlage</h3>
      {boxDid && <p>✅️ Identifikator: {boxDid}</p>}
      {!boxDid && (
        <p>
          ❌️ Identifikator: nicht vorhanden,
          {' '}
          {!boxDidPending && (
            <button
              type="button"
              onClick={handleCreateBoxDIDClick}
              disabled={ownerDidPending}
            >
              erstellen
            </button>
          )}
          {boxDidPending && <progress max={40} value={progress} />}
        </p>
      )}
    </section>

    <section>
      <h3>Betreiber</h3>
      {address && (
        <p>
          {Object.entries(extensions).length === 0 && '❌️ KILT Wallet nicht vorhanden'}

          {!ownerDidPending && <Fragment>
            {Object.entries(extensions).map(([key, { name, getSignedDidCreationExtrinsic }]) => getSignedDidCreationExtrinsic && (
              <button
                type="button"
                key={key}
                name={key}
                onClick={handleCreateOwnerDIDClick}
                disabled={boxDidPending}
              >
                Identifikator erstellen mit {name}
              </button>
            ))}
          </Fragment>}

          {ownerDidPending && <progress max={40} value={progress} />}
        </p>
      )}
    </section>

    <img src={oliLogo} alt="OLI logo" width={116} height={76} className="oli" />
    <img src={kiltLogo} alt="Built on KILT" width={71} height={14} className="kilt" />
  </section>;
}
