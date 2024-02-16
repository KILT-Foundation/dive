import ky from "ky";
import type {
    DidUri,
    KiltAddress,
    IClaimContents,
} from "@kiltprotocol/types";
import { AttestationResponse } from "../types";

export const API_URL = "http://localhost:3333/api/v1";

export async function getPaymentAddress() {

    let response = await ky
        .get(`${API_URL}/payment`);

    if (response.status !== 200) {
        return undefined;
    }

    const { address } =
        await response.json<{ address: KiltAddress }>();


    return address;
}

export async function getExistingDid() {

    const response = await ky.get(`${API_URL}/did`);
    if (response.status !== 200) {
        return undefined;
    }
    const { did } = await response.json<{ did: DidUri }>();
    return did;

}

export async function getClaim() {

    const response = await ky
        .get(`${API_URL}/claim`);

    if (response.status !== 200) {
        return undefined;
    }

    let requestedClaim = await response.json<{ claim: IClaimContents }>();

    return requestedClaim.claim.contents;

}

export async function getCredential() {

    let response = await ky.get(`${API_URL}/credential`, { timeout: false });

    if (response.status !== 200) {
        return undefined;
    }

    let data = await response.json<AttestationResponse[]>();

    if (data.length === 0) {
        return undefined;
    }

    let attestation = data[0]
    if (!attestation.approved) {
        return undefined;
    }

    return attestation.credential;

}
