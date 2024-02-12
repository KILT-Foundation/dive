import ky from "ky";
import type {
    DidUri,
    KiltAddress,
    ICredential,
    IClaimContents,
} from "@kiltprotocol/types";

export const API_URL = "/api/v1";

export async function getPaymentAddress() {
    const { address } = await ky
        .get(`${API_URL}/payment`)
        .json<{ address: KiltAddress }>();
    return address;
}

export async function getExistingDid() {
    try {
        const { did } = await ky.get(`${API_URL}/did`).json<{ did: DidUri }>();
        return did;
    } catch (exception) {
        console.error(exception);
        return undefined;
    }
}

export async function getClaim() {
    try {
        const requested_claim = await ky
            .get(`${API_URL}/claim`)
            .json<{ claim: IClaimContents }>();
        return requested_claim.claim.contents;
    } catch (exception) {
        console.error(exception);
        return undefined;
    }
}

export async function getCredential() {
    try {
        let response = await ky.get(`${API_URL}/credential`).json<ICredential>();

        let data = response[0];
        console.log(data, data.approved, data.credential);
        if (!data.approved) {
            return undefined;
        }

        return data.credential;
    } catch (exception) {
        console.error(exception);
        return undefined;
    }
}
