import { useWallet, useConnection } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useEffect, useState } from "react";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

export function WalletInfo() {
  const { publicKey, connected } = useWallet();
  const { connection } = useConnection();
  const [balance, setBalance] = useState<number | null>(null);

  useEffect(() => {
    if (!publicKey || !connected) {
      setBalance(null);
      return;
    }
    let cancelled = false;
    connection.getBalance(publicKey).then((bal) => {
      if (!cancelled) setBalance(bal / LAMPORTS_PER_SOL);
    });
    const id = setInterval(() => {
      connection.getBalance(publicKey).then((bal) => {
        if (!cancelled) setBalance(bal / LAMPORTS_PER_SOL);
      });
    }, 10000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [publicKey, connected, connection]);

  return (
    <div className="flex items-center gap-3">
      {connected && publicKey && balance !== null && (
        <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 rounded-lg bg-muted text-sm text-muted-foreground">
          <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
          <span className="font-mono text-foreground font-medium">{balance.toFixed(3)} SOL</span>
          <span className="text-xs">Devnet</span>
        </div>
      )}
      <WalletMultiButton
        style={{
          height: "36px",
          fontSize: "13px",
          borderRadius: "8px",
          padding: "0 16px",
          background: "hsl(var(--primary))",
          color: "hsl(var(--primary-foreground))",
        }}
      />
    </div>
  );
}
