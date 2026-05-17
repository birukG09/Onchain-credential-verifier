-- Users (wallet-based accounts)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(42) UNIQUE NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    nonce VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Authorized credential issuers
CREATE TABLE IF NOT EXISTS issuers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(42) UNIQUE NOT NULL,
    organization VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Credentials (NFTs)
CREATE TABLE IF NOT EXISTS credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_id VARCHAR(78),
    issuer_id UUID REFERENCES issuers(id) ON DELETE SET NULL,
    holder_address VARCHAR(42) NOT NULL,
    credential_type VARCHAR(50) NOT NULL,
    title VARCHAR(255),
    metadata_uri TEXT,
    ipfs_hash VARCHAR(100),
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- On-chain transaction records
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tx_hash VARCHAR(66),
    credential_id UUID REFERENCES credentials(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    from_address VARCHAR(42),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Verification audit logs
CREATE TABLE IF NOT EXISTS verification_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    credential_id UUID REFERENCES credentials(id) ON DELETE SET NULL,
    verifier_address VARCHAR(42),
    result BOOLEAN NOT NULL,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common lookups
CREATE INDEX IF NOT EXISTS idx_credentials_holder ON credentials(holder_address);
CREATE INDEX IF NOT EXISTS idx_credentials_issuer ON credentials(issuer_id);
CREATE INDEX IF NOT EXISTS idx_transactions_credential ON transactions(credential_id);
CREATE INDEX IF NOT EXISTS idx_verification_logs_credential ON verification_logs(credential_id);
