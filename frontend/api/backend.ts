import ky, { HTTPError } from "ky";
import type {
    DidUri,
    KiltAddress,
    IClaimContents,
    ICredential,
    IClaim,
} from "@kiltprotocol/types";
import { AttestationResponse } from "../types";

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

export async function getClaim() {
    try {
        const response = await ky.get(`${API_URL}/claim`);

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

        // we are currently only supporting a single credential. Has to be changed once the olibox is able to hold multiple.
        let requestedCredential = data[0];

        if (!requestedCredential.approved) {
            return undefined;
        }

        return requestedCredential.credential;
    } catch (exception) {
        if ((exception as HTTPError).response.status === 404) {
            return undefined;
        }
        console.error(exception);
        throw exception;
    }
}

export async function postClaim(claim: ICredential) {
    try {
        const response = await ky.post(`${API_URL}/claim`, {
            json: claim,
        });
        const data = await response.json<{ claim: IClaim }>();
        return data.claim;
    } catch (exception) {
        console.error(exception);
        throw exception;
    }
}

export async function postUseCaseParticipation(useCaseDidUrl: string, useCaseUrl: string, updateServiceEndpoint: boolean, notifyUseCase: boolean) {
    try {
        const response = await ky.post(`${API_URL}/use-case`, {
            json: { useCaseDidUrl, useCaseUrl, updateServiceEndpoint, notifyUseCase },
        });
    } catch (exception) {
        console.error(exception);
        throw exception;
    }
} 
