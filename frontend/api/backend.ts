import ky from "ky";
import type {
    DidUri,
    KiltAddress,
    ICredential,
    IClaimContents,
} from "@kiltprotocol/types";

export const API_URL = "http://localhost:3333/api/v1";

export async function getPaymentAddress() {
    const { address } = await ky
        .get(`${API_URL}/payment`)
        .json<{ address: KiltAddress }>();


    return address;
}

export async function getExistingDid() {
    try {
        const response = await ky.get(`${API_URL}/did`);
        if (response.status === 404) {
            return undefined;
        }
        const { did } = await response.json<{ did: DidUri }>();
        return did;
    } catch (exception) {
        console.error(exception);
        return undefined;
    }
}

export async function getClaim() {
    try {
        const response = await ky
            .get(`${API_URL}/claim`);

        if (response.status === 404) {
            return undefined;
        }

        let requestedClaim = await response.json<{ claim: IClaimContents }>();

        return requestedClaim.claim.contents;
    } catch (exception) {
        return undefined;
    }
}

export async function getCredential() {
    try {
        let response = await ky.get(`${API_URL}/credential`, { timeout: false }).json<ICredential>();

        let data = response[0];
        if (!data.approved) {
            return undefined;
        }

        return data.credential;
    } catch (exception) {
        console.error(exception);
        return undefined;
    }
}
