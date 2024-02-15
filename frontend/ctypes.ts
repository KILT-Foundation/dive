import type { ICType } from "@kiltprotocol/types";

export const certificateCtype = {
    $id: "kilt:ctype:0xc945ec1d1bc96dcef2c6f1198047c2be7edf7beb5f82c418a19c6614033c6256",
    $schema:
        "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
    additionalProperties: false,
    properties: {
        "Art der Anlage": {
            type: "string",
        },
        "Nennleistung (kW)": {
            type: "number",
        },
        Standort: {
            type: "string",
        },
    },
    title: "DIVE Anlagenzertifikat",
    type: "object",
} as ICType;


export const selfIssuedCtype = {
    "$id": "kilt:ctype:0x707806fa456431dc285a57dbb06258709ee9dad517cbd98a856bb83a57f19a28",
    "$schema": "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
    "additionalProperties": false,
    "properties": {
        "address": {
            "type": "string"
        },
        "name": {
            "type": "string"
        }
    },
    "title": "Selbstauskunfts Zertifikat",
    "type": "object"
} as ICType;
