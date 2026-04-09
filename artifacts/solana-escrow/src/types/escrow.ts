import { PublicKey } from "@solana/web3.js";

export enum EscrowStatus {
  Pending = "Pending",
  Completed = "Completed",
  Cancelled = "Cancelled",
}

export interface EscrowAccount {
  id: string;
  pda: PublicKey;
  buyer: PublicKey;
  seller: PublicKey;
  amount: number;
  amountInSol: number;
  status: EscrowStatus;
  escrowId: number;
}
