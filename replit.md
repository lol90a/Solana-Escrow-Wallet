# Workspace

## Overview

pnpm workspace monorepo. Contains a Solana Escrow DApp with a Rust/Axum backend and React frontend.

## Stack

- **Monorepo tool**: pnpm workspaces
- **Node.js version**: 24
- **Package manager**: pnpm
- **TypeScript version**: 5.9

## Artifacts

### SolEscrow — Solana Escrow DApp (`artifacts/solana-escrow`)
- **Type**: React + Vite frontend
- **Preview path**: `/`
- **Network**: Solana Devnet
- **Wallet**: Phantom wallet via `@solana/wallet-adapter-react`
- **Key deps**: `@solana/web3.js`, `@coral-xyz/anchor`, `@solana/wallet-adapter-*`

Features:
- Connect Phantom wallet (Devnet)
- Create escrow: locks SOL into a PDA, persists record in Rust backend
- View all your escrows with real-time status (Pending / Completed / Cancelled)
- Release funds to seller (Solana tx + Rust API update)
- Cancel escrow to get refunded (Rust API update)
- PDA seeds: `["escrow", buyer_pubkey, escrow_id_u64_le]`

### Rust API Backend (`artifacts/rust-api`)
- **Language**: Rust (stable)
- **Framework**: Axum 0.7
- **Database**: PostgreSQL via tokio-postgres + deadpool-postgres
- **Port**: 8080 (same as api-server artifact)
- **Dev command**: `cargo run --manifest-path /home/runner/workspace/artifacts/rust-api/Cargo.toml`

REST Endpoints:
- `GET /api/healthz` — health check
- `GET /api/escrows?buyer=<pubkey>` — list escrows for a buyer
- `POST /api/escrows` — create escrow record
- `PATCH /api/escrows/:id/release` — mark escrow as Completed
- `PATCH /api/escrows/:id/cancel` — mark escrow as Cancelled
- `DELETE /api/escrows/:id` — delete escrow (dev only)

### On-chain Anchor Program (`artifacts/solana-escrow/program/`)
- **Language**: Rust with Anchor framework
- **Program ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`
- **Network**: Solana Devnet
- Instructions: `create_escrow`, `release_funds`, `cancel_escrow`
- Tests: `program/tests/escrow.ts`

## Key Commands

- `pnpm run typecheck` — full typecheck
- `cargo build` (from `artifacts/rust-api/`) — build Rust backend
- `cargo run` (from `artifacts/rust-api/`) — run Rust backend

## Deploying the Anchor Program

From `artifacts/solana-escrow/program/`:
```bash
solana config set --url devnet
anchor build
anchor deploy --provider.cluster devnet
```
