import { useEscrow } from "@/hooks/useEscrow";
import { CreateEscrowForm } from "@/components/CreateEscrowForm";
import { EscrowCard } from "@/components/EscrowCard";
import { EscrowAccount } from "@/types/escrow";
import { useWallet } from "@solana/wallet-adapter-react";
import { useEffect } from "react";

export function EscrowPage() {
  const { publicKey } = useWallet();
  const { escrows, loading, error, addEscrow, releaseFunds, cancelEscrow, clearError } = useEscrow();

  useEffect(() => {
    if (error) {
      const t = setTimeout(clearError, 5000);
      return () => clearTimeout(t);
    }
  }, [error, clearError]);

  const handleCreate = async (seller: string, amount: number) => {
    await addEscrow(seller, amount);
  };

  const handleRelease = async (escrow: EscrowAccount) => {
    await releaseFunds(escrow);
  };

  const handleCancel = async (escrow: EscrowAccount) => {
    await cancelEscrow(escrow);
  };

  const myEscrows = escrows.filter(
    (e) => publicKey && e.buyer.toBase58() === publicKey.toBase58()
  );

  return (
    <div className="min-h-screen bg-background">
      <div className="max-w-4xl mx-auto px-4 py-8 space-y-8">
        {error && (
          <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 text-sm text-destructive flex items-center justify-between">
            <span>{error}</span>
            <button onClick={clearError} className="ml-3 text-destructive hover:opacity-70 transition-opacity">
              ✕
            </button>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-1">
            <CreateEscrowForm onSubmit={handleCreate} loading={loading} />

            <div className="mt-4 p-4 bg-muted/50 rounded-xl border border-border text-xs text-muted-foreground space-y-2">
              <p className="font-medium text-foreground text-sm">How It Works</p>
              <div className="space-y-1.5">
                <div className="flex gap-2">
                  <span className="text-primary font-bold">1.</span>
                  <span>Connect your Phantom wallet (Devnet)</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-primary font-bold">2.</span>
                  <span>Enter the seller's address and SOL amount</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-primary font-bold">3.</span>
                  <span>Funds are locked in a PDA-backed escrow</span>
                </div>
                <div className="flex gap-2">
                  <span className="text-primary font-bold">4.</span>
                  <span>Release to pay seller or cancel to refund yourself</span>
                </div>
              </div>
            </div>

            <div className="mt-4 p-4 bg-muted/30 rounded-xl border border-border text-xs text-muted-foreground">
              <p className="font-medium text-foreground text-sm mb-2">On-Chain Program</p>
              <p className="mb-1">Network: <span className="text-foreground font-mono">Devnet</span></p>
              <p>PDA seeds: <span className="font-mono text-foreground">[&quot;escrow&quot;, buyer, id]</span></p>
            </div>
          </div>

          <div className="lg:col-span-2">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-foreground">Your Escrows</h2>
              {myEscrows.length > 0 && (
                <span className="text-sm text-muted-foreground">{myEscrows.length} total</span>
              )}
            </div>

            {!publicKey ? (
              <div className="flex flex-col items-center justify-center h-48 text-center bg-muted/20 rounded-xl border border-dashed border-border">
                <p className="text-muted-foreground text-sm">Connect your wallet to see escrows</p>
              </div>
            ) : myEscrows.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-48 text-center bg-muted/20 rounded-xl border border-dashed border-border">
                <p className="text-muted-foreground text-sm">No escrows yet</p>
                <p className="text-muted-foreground text-xs mt-1">Create your first escrow to get started</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                {myEscrows.map((escrow) => (
                  <EscrowCard
                    key={escrow.id}
                    escrow={escrow}
                    onRelease={handleRelease}
                    onCancel={handleCancel}
                    loading={loading}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
