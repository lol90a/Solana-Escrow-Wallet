# Solana Escrow Wallet

Rust workspace for a Solana escrow system with:

- `apps/escrow-api`: Axum HTTP API
- `programs/escrow-program`: Anchor on-chain program

There is no active frontend in the repository right now. The previous TypeScript/React workspace was removed during the Rust-only refactor.

## Current Status

- Rust workspace: active
- API backend: active
- Anchor program: active
- Frontend UI: not present

## Architecture

The API follows clean architecture boundaries:

- `domain`: escrow entities and core business rules
- `application`: use cases and repository contracts
- `infrastructure`: Postgres persistence and migrations
- `presentation`: HTTP routing, request parsing, and response mapping

This keeps business logic independent from Axum and Postgres details.

## Workspace Layout

```text
.
|-- Cargo.toml
|-- apps
|   `-- escrow-api
|       `-- src
|           |-- application
|           |-- domain
|           |-- infrastructure
|           |-- presentation
|           |-- config.rs
|           |-- lib.rs
|           `-- main.rs
`-- programs
    `-- escrow-program
        |-- src
        `-- tests
```

## Requirements

- Rust stable
- Cargo
- PostgreSQL for the API
- Solana CLI and Anchor CLI for program development

## Environment

The API expects:

```bash
DATABASE_URL=postgres://...
PORT=8080
DATABASE_POOL_SIZE=10
```

## Running

Run the API:

```bash
cargo run -p escrow-api
```

Check the Anchor program:

```bash
cargo check -p escrow-program
```

Run the full workspace tests:

```bash
cargo test --workspace
```

## API Endpoints

- `GET /api/healthz`
- `GET /api/escrows?buyer=<pubkey>`
- `POST /api/escrows`
- `PATCH /api/escrows/:id/release`
- `PATCH /api/escrows/:id/cancel`
- `DELETE /api/escrows/:id`

## Testing

The workspace currently has Rust-native tests for:

- API service behavior
- HTTP health route wiring
- Anchor account sizing
- Anchor enum serialization

Latest local verification completed with:

```bash
cargo test --workspace
```

