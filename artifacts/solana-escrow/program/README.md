# Solana Escrow Program (Anchor)

This is the on-chain Anchor/Rust smart contract for the SolEscrow DApp.

## Prerequisites

- Rust: https://www.rust-lang.org/tools/install
- Solana CLI: https://docs.solanalabs.com/cli/install
- Anchor CLI: https://www.anchor-lang.com/docs/installation

## Program Overview

The escrow program uses a **Program Derived Address (PDA)** to securely hold funds between a buyer and seller.

### PDA Seeds
```
["escrow", buyer_pubkey, escrow_id (u64 LE bytes)]
```

### Instructions

| Instruction | Signer | Description |
|---|---|---|
| `create_escrow` | buyer | Locks SOL into a PDA escrow account |
| `release_funds` | buyer | Transfers locked SOL to the seller |
| `cancel_escrow` | buyer | Refunds locked SOL back to the buyer |

### Account Structure

```rust
pub struct EscrowAccount {
    pub buyer: Pubkey,        // Who created the escrow
    pub seller: Pubkey,       // Who receives the funds
    pub amount: u64,          // Lamports locked
    pub status: EscrowStatus, // Pending | Completed | Cancelled
    pub escrow_id: u64,       // Unique ID (used in PDA derivation)
    pub bump: u8,             // PDA bump seed
}
```

## Setup & Deploy

```bash
# Install dependencies
yarn install

# Configure Solana CLI for devnet
solana config set --url devnet

# Generate a new keypair (if needed)
solana-keygen new

# Airdrop SOL on devnet
solana airdrop 2

# Build the program
anchor build

# Run tests (localnet)
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

## Running Tests

```bash
anchor test
```

Tests cover:
1. Creating an escrow and verifying SOL is locked in the PDA
2. Releasing funds to the seller
3. Cancelling an escrow and verifying the buyer is refunded
4. Preventing unauthorized users from performing actions
