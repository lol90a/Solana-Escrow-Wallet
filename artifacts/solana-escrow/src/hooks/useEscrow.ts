import { useState, useCallback, useEffect } from "react";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { EscrowAccount, EscrowStatus } from "@/types/escrow";

const PROGRAM_ID = new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
const API_BASE = "/api";

function getEscrowPDA(buyer: PublicKey, escrowId: number): [PublicKey, number] {
  const escrowIdBuf = new Uint8Array(8);
  const view = new DataView(escrowIdBuf.buffer);
  view.setBigUint64(0, BigInt(escrowId), true);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), buyer.toBuffer(), escrowIdBuf],
    PROGRAM_ID
  );
}

interface ApiEscrow {
  id: string;
  buyer: string;
  seller: string;
  amount: number;
  amount_sol: number;
  status: string;
  escrow_id: number;
  pda: string;
  created_at: string;
}

function fromApi(e: ApiEscrow): EscrowAccount {
  return {
    id: e.id,
    pda: new PublicKey(e.pda),
    buyer: new PublicKey(e.buyer),
    seller: new PublicKey(e.seller),
    amount: e.amount,
    amountInSol: e.amount_sol,
    status: e.status as EscrowStatus,
    escrowId: e.escrow_id,
  };
}

export function useEscrow() {
  const { publicKey, sendTransaction, connected } = useWallet();
  const { connection } = useConnection();

  const [escrows, setEscrows] = useState<EscrowAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load existing escrows from Rust backend whenever wallet connects
  useEffect(() => {
    if (!publicKey) {
      setEscrows([]);
      return;
    }
    let cancelled = false;
    setLoading(true);
    fetch(`${API_BASE}/escrows?buyer=${publicKey.toBase58()}`)
      .then((r) => r.json())
      .then((res: { success: boolean; data?: ApiEscrow[] }) => {
        if (!cancelled && res.success && res.data) {
          setEscrows(res.data.map(fromApi));
        }
      })
      .catch(() => {
        if (!cancelled) setError("Could not load escrows from server.");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, [publicKey]);

  const addEscrow = useCallback(
    async (sellerAddress: string, amountSol: number) => {
      if (!publicKey || !connected) {
        setError("Please connect your wallet first.");
        return null;
      }
      setLoading(true);
      setError(null);

      try {
        let sellerPubkey: PublicKey;
        try {
          sellerPubkey = new PublicKey(sellerAddress);
        } catch {
          throw new Error("Invalid seller address.");
        }

        if (amountSol <= 0) throw new Error("Amount must be greater than 0 SOL.");

        const escrowId = Date.now();
        const [escrowPDA] = getEscrowPDA(publicKey, escrowId);
        const lamports = Math.floor(amountSol * LAMPORTS_PER_SOL);

        // Build and send SOL transfer transaction to lock funds in PDA
        const tx = new Transaction().add(
          SystemProgram.transfer({
            fromPubkey: publicKey,
            toPubkey: escrowPDA,
            lamports,
          })
        );

        tx.feePayer = publicKey;
        const { blockhash } = await connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;

        const sig = await sendTransaction(tx, connection);
        await connection.confirmTransaction(sig, "confirmed");

        // Persist escrow record in Rust backend
        const res = await fetch(`${API_BASE}/escrows`, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            id: sig,
            buyer: publicKey.toBase58(),
            seller: sellerPubkey.toBase58(),
            amount: lamports,
            amount_sol: amountSol,
            escrow_id: escrowId,
            pda: escrowPDA.toBase58(),
          }),
        });

        const json = await res.json() as { success: boolean; data?: ApiEscrow; message?: string };
        if (!json.success || !json.data) {
          throw new Error(json.message || "Failed to save escrow to server.");
        }

        const newEscrow = fromApi(json.data);
        setEscrows((prev) => [newEscrow, ...prev]);
        return newEscrow;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : "Transaction failed.";
        setError(msg);
        return null;
      } finally {
        setLoading(false);
      }
    },
    [publicKey, connected, connection, sendTransaction]
  );

  const releaseFunds = useCallback(
    async (escrow: EscrowAccount) => {
      if (!publicKey || !connected) {
        setError("Please connect your wallet first.");
        return false;
      }
      if (escrow.buyer.toBase58() !== publicKey.toBase58()) {
        setError("Only the buyer can release funds.");
        return false;
      }
      if (escrow.status !== EscrowStatus.Pending) {
        setError("Escrow is not in pending status.");
        return false;
      }
      setLoading(true);
      setError(null);

      try {
        // Transfer SOL to seller
        const tx = new Transaction().add(
          SystemProgram.transfer({
            fromPubkey: publicKey,
            toPubkey: escrow.seller,
            lamports: escrow.amount,
          })
        );

        tx.feePayer = publicKey;
        const { blockhash } = await connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;

        await sendTransaction(tx, connection);

        // Update status in Rust backend
        const res = await fetch(`${API_BASE}/escrows/${escrow.id}/release`, {
          method: "PATCH",
        });
        const json = await res.json() as { success: boolean; data?: ApiEscrow; message?: string };

        if (!json.success) throw new Error(json.message || "Failed to update status.");

        setEscrows((prev) =>
          prev.map((e) =>
            e.id === escrow.id ? { ...e, status: EscrowStatus.Completed } : e
          )
        );
        return true;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : "Release failed.";
        setError(msg);
        return false;
      } finally {
        setLoading(false);
      }
    },
    [publicKey, connected, connection, sendTransaction]
  );

  const cancelEscrow = useCallback(
    async (escrow: EscrowAccount) => {
      if (!publicKey || !connected) {
        setError("Please connect your wallet first.");
        return false;
      }
      if (escrow.buyer.toBase58() !== publicKey.toBase58()) {
        setError("Only the buyer can cancel the escrow.");
        return false;
      }
      if (escrow.status !== EscrowStatus.Pending) {
        setError("Escrow is not in pending status.");
        return false;
      }
      setLoading(true);
      setError(null);

      try {
        // Update status in Rust backend
        const res = await fetch(`${API_BASE}/escrows/${escrow.id}/cancel`, {
          method: "PATCH",
        });
        const json = await res.json() as { success: boolean; data?: ApiEscrow; message?: string };

        if (!json.success) throw new Error(json.message || "Failed to update status.");

        setEscrows((prev) =>
          prev.map((e) =>
            e.id === escrow.id ? { ...e, status: EscrowStatus.Cancelled } : e
          )
        );
        return true;
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : "Cancel failed.";
        setError(msg);
        return false;
      } finally {
        setLoading(false);
      }
    },
    [publicKey, connected]
  );

  const clearError = useCallback(() => setError(null), []);

  return {
    escrows,
    loading,
    error,
    addEscrow,
    releaseFunds,
    cancelEscrow,
    clearError,
  };
}
