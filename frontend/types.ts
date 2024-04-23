import type { ICredential } from "@kiltprotocol/types";

export interface AttestationResponse {
  id: string;
  approved: boolean;
  revoked: boolean;
  marked_approve: boolean;
  created_at: string;
  deleted_at: string | null;
  approved_at: string | null;
  revoked_at: string | null;
  ctype_hash: string;
  credential: ICredential;
  claimer: string;
  tx_state: number | null;
}

export interface UseCaseResponse {
  useCase: string;
}

export interface UseCaseConfig {
  useCaseDidUrl: string;
  useCaseUrl: string;
  updateServiceEndpoint: boolean;
  notifyUseCase: boolean;
}
