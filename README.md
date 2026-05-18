# On-Chain Credential Verifier

Blockchain-backed credential verification. Mints ERC721 NFTs for education, certifications, and employment history — verifiable on Polygon.

## Stacks

- **Backend** — Rust + Axum 0.8
- **Database** — PostgreSQL (sqlx)
- **Smart Contract** — Solidity 0.8.24 + OpenZeppelin ERC721
- **Contract Tooling** — Hardhat

## Project Layout

```
artifacts/api-server/     # Rust/Axum REST API
  src/
    main.rs               # Entry point
    routes/               # HTTP handlers
    services/             # Business logic
    db/                   # SQL queries
    blockchain/           # JSON-RPC client
    auth/                 # JWT + wallet sig
  migrations/             # PostgreSQL schema
  Cargo.toml

contracts/                # Hardhat project
  contracts/
    CredentialNFT.sol     # ERC721 contract
  scripts/deploy.ts       # Deploy to Mumbai
  test/CredentialNFT.test.ts
```

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/healthz` | Health check |
| POST | `/api/issuer/register` | Register an issuer |
| GET | `/api/issuer/list` | List active issuers |
| POST | `/api/credentials/mint` | Mint a credential |
| GET | `/api/credentials/{id}` | Get credential by ID |
| POST | `/api/credentials/verify` | Verify authenticity |
| POST | `/api/credentials/revoke` | Revoke a credential |

## Running

```bash
# Start API (compiles Rust on first run, ~30s)
pnpm --filter @workspace/api-server run dev

# Compile contracts
cd contracts && npx hardhat compile

# Run contract tests
cd contracts && npx hardhat test
```

## Deploy to Polygon Mumbai

```bash
# 1. Set env vars
export ISSUER_PRIVATE_KEY=<your-private-key>
export POLYGON_RPC_URL=<mumbai-rpc-url>

# 2. Deploy
cd contracts && npx hardhat run scripts/deploy.ts --network mumbai

# 3. Set CONTRACT_ADDRESS env var with the deployed address
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `JWT_SECRET` | Recommended | JWT signing secret |
| `CONTRACT_ADDRESS` | Optional | Deployed contract address |
| `POLYGON_RPC_URL` | Optional | Polygon Mumbai RPC |
| `ISSUER_PRIVATE_KEY` | Optional | Issuer wallet private key |
