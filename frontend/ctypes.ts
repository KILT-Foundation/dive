import type { ICType } from "@kiltprotocol/types";

export const productionCtype = {
  $id: "kilt:ctype:0xc49c00d77f39945b665f4cd85b8953e73292131046c01eb696440ecb851c68e8",
  $schema:
    "ipfs://bafybeiah66wbkhqbqn7idkostj2iqyan2tstc4tpqt65udlhimd7hcxjyq/",
  additionalProperties: false,
  properties: {
    Arbeitsvermögensbegrenzung: {
      type: "string",
    },
    "Bezeichnung (Anbieterintern)": {
      type: "string",
    },
    "BNetzA-Kraftwerksnummer": {
      type: "string",
    },
    "CO2-Äquivalent-Emission": {
      type: "string",
    },
    "E-Mail": {
      type: "string",
    },
    "EEG-Anlagenschlüssel": {
      type: "string",
    },
    "EIC-W der Einheit": {
      type: "string",
    },
    Geburtsdatum: {
      format: "date",
      type: "string",
    },
    "Marktlokations-ID": {
      type: "string",
    },
    "Marktstammdatenregister-ID": {
      type: "string",
    },
    "Maximal-Leistung": {
      type: "number",
    },
    "Messlokations-ID": {
      type: "string",
    },
    "Meter ID": {
      type: "string",
    },
    "Minimal-Leistung": {
      type: "number",
    },
    Nachname: {
      type: "string",
    },
    "Nennleistung (W)": {
      type: "string",
    },
    "Obere Grenze der Leistungsregelung (W)": {
      type: "number",
    },
    Postleitzahl: {
      type: "string",
    },
    Primärenergieträger: {
      type: "string",
    },
    "SMGW ID": {
      type: "string",
    },
    Spannungsebene: {
      type: "string",
    },
    Standort: {
      type: "string",
    },
    Steuernummer: {
      type: "string",
    },
    "Straße und Hausnummer": {
      type: "string",
    },
    Technologie: {
      type: "string",
    },
    Telefonnummer: {
      type: "string",
    },
    Umsatzsteueridentifikationsnummer: {
      type: "string",
    },
    "untere Grenze der Leistungsregelung": {
      type: "number",
    },
    Unternehmenstyp: {
      type: "string",
    },
    Verteilnetzbetreiber: {
      type: "string",
    },
    Vorname: {
      type: "string",
    },
    "Zeitpunkt der Installation": {
      format: "date",
      type: "string",
    },
  },
  title: "Basis Dive Anlagezertifikat",
  type: "object",
} as ICType;

export const presentationCtype = {
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
