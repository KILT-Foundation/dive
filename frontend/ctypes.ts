import type { ICType } from "@kiltprotocol/types";

export const cType = {
  $id: "kilt:ctype:0x2a63756ff4934eb51d5c405476ea92dfa9413388a8a33c37755442e2111304b5",
  $schema:
    "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
  additionalProperties: false,
  properties: {
    Anschlussnetzbetreiber: {
      type: "string",
    },
    "Art der Anlage": {
      type: "string",
    },
    Betreiber: {
      type: "string",
    },
    Betreiberstatus: {
      type: "string",
    },
    Bruttoleistung: {
      type: "number",
    },
    "EEG Inbetriebnahmedatum": {
      format: "date",
      type: "string",
    },
    "EEG Registrierungsdatum": {
      format: "date",
      type: "string",
    },
    "Errichtungsort (Lage)": {
      type: "string",
    },
    Inbetriebnahmedatum: {
      format: "date",
      type: "string",
    },
    "Installierte Leistung": {
      type: "number",
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
    "Name der Einheit": {
      type: "string",
    },
    "Registrierungsdatum im aktuellen Betriebsstatus": {
      format: "date",
      type: "string",
    },
    "SMGW ID": {
      type: "string",
    },
    Standort: {
      type: "string",
    },
    Wechselrichterleistung: {
      type: "number",
    },
  },
  title: "DIVE Anlagezertifikat",
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
