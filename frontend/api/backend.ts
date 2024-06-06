import ky, { HTTPError } from "ky";
import type {
  DidUri,
  KiltAddress,
  IClaimContents,
  ICredential,
  IClaim,
} from "@kiltprotocol/types";
import { AttestationResponse, UseCaseConfig, UseCaseResponse } from "../types";

const basePrefixUrl = process.env.API_URL || '/'
const prefixUrl = `${basePrefixUrl}api/v1/`;

export const baseApi = ky.extend({ prefixUrl: basePrefixUrl });
export const api = ky.extend({ prefixUrl });

export async function getPaymentAddress() {
  try {
    const response = await api.get(`payment`);
    const { address } = await response.json<{ address: KiltAddress }>();
    return address;
  } catch (exception) {
    if ((exception as HTTPError).response.status !== 200) {
      return undefined;
    }
  }
}

export async function getExistingDid() {
  try {
    const response = await api.get(`did`);
    const { did } = await response.json<{ did: DidUri }>();
    return did;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return undefined;
    }
    throw exception;
  }
}

export async function getClaim() {
  try {
    const response = await api.get(`claim`);

    const requestedClaim = await response.json<{ claim: IClaimContents }>();

    return requestedClaim.claim.contents;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return undefined;
    }
    console.error(exception);
    throw exception;
  }
}

export async function getCredential() {
  try {
    const response = await api.get(`credential`, {
      timeout: false,
    });
    const data = await response.json<AttestationResponse[]>();

    if (data.length === 0) {
      return [];
    }

    return data;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return [];
    }
    console.error(exception);
    throw exception;
  }
}

export async function postClaim(claim: ICredential) {
  try {
    const response = await api.post(`claim`, {
      json: claim,
    });
    const data = await response.json<{ claim: IClaim }>();
    return data.claim;
  } catch (exception) {
    console.error(exception);
    throw exception;
  }
}

export async function postUseCaseParticipation(useCaseConfig: UseCaseConfig) {
  try {
    const response = await api.post(`use-case`, {
      json: useCaseConfig,
      timeout: false,
    });

    return await response.json<string>();
  } catch (exception) {
    console.error(exception);
    throw exception;
  }
}

export async function getActiveUseCase() {
  try {
    const response = await api.get(`use-case`, {
      timeout: false,
    });
    const data = await response.json<UseCaseResponse>();

    return data.useCase;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return undefined;
    }
    console.error(exception);
    throw exception;
  }
}

// this function is needed because of the credential api.
// in the olibox setup, the frontend is served via an proxy.
// each session can have a different, which would result into an error in the credential api flow
// This function is updating the url in the did-configuration.json for the backend.
// SUUUUPER UGLY. But it works for now. :)
export async function postUrl() {
  try {
    await baseApi.post(`.well-known/did-configuration.json`, {
      json: {
        url: window.location.origin,
      },
    });
  } catch (exception) {
    console.error(exception);
    throw exception;
  }
}
