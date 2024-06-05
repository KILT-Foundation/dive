import type { ICType } from "@kiltprotocol/types";

export const cType = {
  $id: "kilt:ctype:0xbde8192c1a7e67395218029a7028af448505238c85123cb8a4fbe274bf45f71f",
  $schema:
    "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
  additionalProperties: false,
  properties: {
    "Art der Anlage": {
      type: "string",
    },
    "Marktlokations-ID": {
      type: "string",
    },
    "Messlokations-ID": {
      type: "string",
    },
    "Meter ID": {
      type: "string",
    },
    "Nennleistung (kW)": {
      type: "number",
    },
    "SMGW ID": {
      type: "string",
    },
    Standort: {
      type: "string",
    },
  },
  title: "DIVE Anlagenzertifikat",
  type: "object",
} as ICType;

export const selfIssuedCtype = {
  $id: "kilt:ctype:0x707806fa456431dc285a57dbb06258709ee9dad517cbd98a856bb83a57f19a28",
  $schema:
    "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
  additionalProperties: false,
  properties: {
    address: {
      type: "string",
    },
    name: {
      type: "string",
    },
  },
  title: "Selbstauskunfts Zertifikat",
  type: "object",
} as ICType;
