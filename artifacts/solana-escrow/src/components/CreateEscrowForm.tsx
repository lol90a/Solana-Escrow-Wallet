import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";

interface Props {
  onSubmit: (seller: string, amount: number) => Promise<void>;
  loading: boolean;
}

export function CreateEscrowForm({ onSubmit, loading }: Props) {
  const { connected } = useWallet();
  const [seller, setSeller] = useState("");
  const [amount, setAmount] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const amountNum = parseFloat(amount);
    if (!seller.trim() || isNaN(amountNum) || amountNum <= 0) return;
    await onSubmit(seller.trim(), amountNum);
    setSeller("");
    setAmount("");
  };

  return (
    <div className="bg-card border border-card-border rounded-xl p-6">
      <h2 className="text-lg font-semibold text-foreground mb-4">Create Escrow</h2>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-muted-foreground mb-1">
            Seller Address
          </label>
          <input
            type="text"
            value={seller}
            onChange={(e) => setSeller(e.target.value)}
            placeholder="Solana public key (base58)"
            disabled={!connected || loading}
            className="w-full px-3 py-2 rounded-lg bg-background border border-input text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50 disabled:cursor-not-allowed font-mono"
          />
        </div>

        <div>
          <label className="block text-sm font-medium text-muted-foreground mb-1">
            Amount (SOL)
          </label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="0.00"
            min="0.001"
            step="0.001"
            disabled={!connected || loading}
            className="w-full px-3 py-2 rounded-lg bg-background border border-input text-foreground text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring disabled:opacity-50 disabled:cursor-not-allowed"
          />
        </div>

        <button
          type="submit"
          disabled={!connected || loading || !seller || !amount}
          className="w-full py-2.5 px-4 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? "Processing..." : connected ? "Lock Funds in Escrow" : "Connect Wallet First"}
        </button>
      </form>
    </div>
  );
}
