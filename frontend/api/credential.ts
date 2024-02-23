import { PubSubSessionV1, PubSubSessionV2 } from '@kiltprotocol/kilt-extension-api'
import type { IClaim } from "@kiltprotocol/types"
import ky from 'ky';

export async function fetchCredential(session: PubSubSessionV1 | PubSubSessionV2, claim: IClaim) {
  const credentialUrl = `api/v1/credential`;


  const getTermsResponse = await ky.post(credentialUrl + "/terms", { json: claim });

  if (getTermsResponse.status !== 200) {
    throw Error("Failed to get terms", await getTermsResponse.json())
  }

  const data = await getTermsResponse.json();


  const getCredentialRequestFromExtension = await new Promise((resolve, reject) => {
    try {
      session.listen(async (credentialRequest: unknown) => {
        resolve(credentialRequest)
      })
      session.send(data)
    } catch (e) {
      reject(e)
    }
  })

  const credentialResponse = await ky.post(credentialUrl, { json: getCredentialRequestFromExtension, timeout: false });

  if (credentialResponse.status !== 200) {
    throw Error("Failed to get terms", await credentialResponse.json())
  }

}
