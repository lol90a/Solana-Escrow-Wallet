import { useState, useCallback } from "react";
import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import {
  PublicKey,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { EscrowAccount, EscrowStatus } from "@/types/escrow";

const PROGRAM_ID = new PublicKey("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

function getEscrowPDA(buyer: PublicKey, escrowId: number): [PublicKey, number] {
  const escrowIdBuf = Buffer.alloc(8);
  escrowIdBuf.writeBigUInt64LE(BigInt(escrowId));
  return PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), buyer.toBuffer(), escrowIdBuf],
    PROGRAM_ID
  );
}

export function useEscrow() {
  const { publicKey, sendTransaction, connected } = useWallet();
  const { connection } = useConnection();

  const [escrows, setEscrows] = useState<EscrowAccount[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
        const [escrowPDA, bump] = getEscrowPDA(publicKey, escrowId);
        const lamports = Math.floor(amountSol * LAMPORTS_PER_SOL);

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

        const newEscrow: EscrowAccount = {
          id: sig,
          pda: escrowPDA,
          buyer: publicKey,
          seller: sellerPubkey,
          amount: lamports,
          amountInSol: amountSol,
          status: EscrowStatus.Pending,
          escrowId,
        };

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
        const balance = await connection.getBalance(escrow.pda);
        if (balance === 0) {
          setEscrows((prev) =>
            prev.map((e) =>
              e.id === escrow.id ? { ...e, status: EscrowStatus.Completed } : e
            )
          );
          return true;
        }

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

        const sig = await sendTransaction(tx, connection);
        await connection.confirmTransaction(sig, "confirmed");

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
