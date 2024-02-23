import { InjectedWindowProvider, PubSubSessionV1, PubSubSessionV2 } from '@kiltprotocol/kilt-extension-api'
import ky from 'ky';

export async function getSession(provider: InjectedWindowProvider): Promise<PubSubSessionV1 | PubSubSessionV2> {
  if (!provider) {
    throw new Error('No provider')
  }

  const challengeUrl = `api/v1/challenge`;

  const getChallengeReponse = await ky.get(challengeUrl);

  if (getChallengeReponse.status !== 200) {
    throw new Error('No valid challenge received')
  }

  const { dAppName, dAppEncryptionKeyUri, challenge } = await getChallengeReponse.json<any>()


  const session = await provider.startSession(dAppName, dAppEncryptionKeyUri, challenge)


  // post challenge and receive encrypted Message.
  const sessionVerification = await ky.post(challengeUrl, { json: session })

  if (sessionVerification.status !== 200) {
    throw new Error('No valid Session.')
  }

  return session
}
