# Workspace

## Overview

pnpm workspace monorepo using TypeScript. Contains a Solana Escrow DApp as the main artifact.

## Stack

- **Monorepo tool**: pnpm workspaces
- **Node.js version**: 24
- **Package manager**: pnpm
- **TypeScript version**: 5.9
- **API framework**: Express 5
- **Database**: PostgreSQL + Drizzle ORM
- **Validation**: Zod (`zod/v4`), `drizzle-zod`
- **API codegen**: Orval (from OpenAPI spec)
- **Build**: esbuild (CJS bundle)

## Artifacts

### SolEscrow — Solana Escrow DApp (`artifacts/solana-escrow`)
- **Type**: React + Vite frontend
- **Preview path**: `/`
- **Network**: Solana Devnet
- **Wallet**: Phantom wallet via `@solana/wallet-adapter-react`
- **Dependencies**: `@solana/web3.js`, `@coral-xyz/anchor`, `@solana/wallet-adapter-*`

Features:
- Connect Phantom wallet (Devnet)
- Create escrow by locking SOL into a PDA account
- View all your escrows with status (Pending / Completed / Cancelled)
- Release funds to seller or cancel to refund yourself
- PDA seeds: `["escrow", buyer_pubkey, escrow_id_u64_le]`

### On-chain Anchor Program (`artifacts/solana-escrow/program/`)
- **Language**: Rust with Anchor framework
- **Program ID**: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`
- **Network**: Solana Devnet
- Instructions: `create_escrow`, `release_funds`, `cancel_escrow`
- Tests: `program/tests/escrow.ts` (Anchor test suite with Mocha/Chai)

## Key Commands

- `pnpm run typecheck` — full typecheck across all packages
- `pnpm run build` — typecheck + build all packages
- `pnpm --filter @workspace/api-spec run codegen` — regenerate API hooks and Zod schemas from OpenAPI spec
- `pnpm --filter @workspace/db run push` — push DB schema changes (dev only)
- `pnpm --filter @workspace/api-server run dev` — run API server locally

## Deploying the Anchor Program

From `artifacts/solana-escrow/program/`:
```bash
anchor build
anchor test        # localnet
anchor deploy --provider.cluster devnet
```

See the `pnpm-workspace` skill for workspace structure, TypeScript setup, and package details.
