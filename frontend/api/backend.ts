import ky, { HTTPError } from "ky";
import type {
  DidUri,
  KiltAddress,
  IClaimContents,
  ICredential,
  IClaim,
} from "@kiltprotocol/types";
import { AttestationResponse, UseCaseConfig, UseCaseResponse } from "../types";
import { Mode } from "../App";

export const API_URL = "http://localhost:3333/api/v1";

export async function getPaymentAddress() {
  try {
    const response = await ky.get(`${API_URL}/payment`);
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
    const response = await ky.get(`${API_URL}/did`);
    const { did } = await response.json<{ did: DidUri }>();
    return did;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return undefined;
    }
    throw exception;
  }
}

export async function getClaim(mode: Mode) {
  try {
    const response = await ky.get(`${API_URL}/claim/${mode}`);

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
    const response = await ky.get(`${API_URL}/credential`, { timeout: false });
    const data = await response.json<AttestationResponse[]>();

    if (data.length === 0) {
      return undefined;
    }

    const requestedCredentials = data.map((d) => d.credential);

    return requestedCredentials;
  } catch (exception) {
    if ((exception as HTTPError).response.status === 404) {
      return undefined;
    }
    console.error(exception);
    throw exception;
  }
}

export async function postClaim(claim: ICredential, mode: Mode) {
  try {
    const response = await ky.post(`${API_URL}/claim/${mode}`, {
      json: claim,
    });
    const data = await response.json<{ claim: IClaim }>();
    return data.claim;
  } catch (exception) {
    console.error(exception);
    throw exception;
  }
}

export async function postUseCaseParticipation(
  useCaseConfig: UseCaseConfig,
  mode: Mode
) {
  try {
    const response = await ky.post(`${API_URL}/use-case/${mode}`, {
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
    const response = await ky.get(`${API_URL}/use-case`, { timeout: false });
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
