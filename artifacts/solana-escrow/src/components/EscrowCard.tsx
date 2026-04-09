import { useWallet } from "@solana/wallet-adapter-react";
import { EscrowAccount, EscrowStatus } from "@/types/escrow";

interface Props {
  escrow: EscrowAccount;
  onRelease: (escrow: EscrowAccount) => Promise<void>;
  onCancel: (escrow: EscrowAccount) => Promise<void>;
  loading: boolean;
}

const statusConfig = {
  [EscrowStatus.Pending]: {
    label: "Pending",
    className: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400",
  },
  [EscrowStatus.Completed]: {
    label: "Completed",
    className: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400",
  },
  [EscrowStatus.Cancelled]: {
    label: "Cancelled",
    className: "bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400",
  },
};

function truncate(addr: string, chars = 6) {
  return `${addr.slice(0, chars)}...${addr.slice(-chars)}`;
}

export function EscrowCard({ escrow, onRelease, onCancel, loading }: Props) {
  const { publicKey } = useWallet();
  const isBuyer = publicKey?.toBase58() === escrow.buyer.toBase58();
  const isPending = escrow.status === EscrowStatus.Pending;
  const { label, className } = statusConfig[escrow.status];

  return (
    <div className="bg-card border border-card-border rounded-xl p-5 flex flex-col gap-4">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-xs text-muted-foreground mb-0.5">Escrow ID</p>
          <p className="text-sm font-mono text-foreground font-medium">
            #{escrow.escrowId.toString().slice(-8)}
          </p>
        </div>
        <span className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${className}`}>
          {label}
        </span>
      </div>

      <div className="grid grid-cols-2 gap-3 text-sm">
        <div>
          <p className="text-xs text-muted-foreground mb-0.5">Buyer</p>
          <p className="font-mono text-foreground text-xs" title={escrow.buyer.toBase58()}>
            {truncate(escrow.buyer.toBase58())}
          </p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground mb-0.5">Seller</p>
          <p className="font-mono text-foreground text-xs" title={escrow.seller.toBase58()}>
            {truncate(escrow.seller.toBase58())}
          </p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground mb-0.5">Amount</p>
          <p className="font-semibold text-foreground">{escrow.amountInSol.toFixed(3)} SOL</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground mb-0.5">PDA</p>
          <p className="font-mono text-foreground text-xs" title={escrow.pda.toBase58()}>
            {truncate(escrow.pda.toBase58())}
          </p>
        </div>
      </div>

      {isBuyer && isPending && (
        <div className="flex gap-2 pt-1">
          <button
            onClick={() => onRelease(escrow)}
            disabled={loading}
            className="flex-1 py-2 px-3 rounded-lg bg-primary text-primary-foreground text-xs font-medium hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Release Funds
          </button>
          <button
            onClick={() => onCancel(escrow)}
            disabled={loading}
            className="flex-1 py-2 px-3 rounded-lg bg-destructive text-destructive-foreground text-xs font-medium hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Cancel
          </button>
        </div>
      )}

      {!isBuyer && isPending && (
        <p className="text-xs text-muted-foreground italic">
          Awaiting buyer action
        </p>
      )}
    </div>
  );
}
