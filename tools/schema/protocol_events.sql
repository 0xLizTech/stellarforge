-- PostgreSQL Schema for StellarForge Protocol Events (#205)

CREATE TABLE IF NOT EXISTS protocol_events (
    id BIGSERIAL PRIMARY KEY,
    contract_id VARCHAR(56) NOT NULL,
    event_type VARCHAR(64) NOT NULL,
    ledger_sequence INTEGER NOT NULL,
    ledger_closed_at TIMESTAMP WITH TIME ZONE NOT NULL,
    tx_hash VARCHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_protocol_events_contract ON protocol_events(contract_id);
CREATE INDEX IF NOT EXISTS idx_protocol_events_ledger ON protocol_events(ledger_sequence);
CREATE INDEX IF NOT EXISTS idx_protocol_events_type ON protocol_events(event_type);

CREATE TABLE IF NOT EXISTS vault_positions (
    vault_id VARCHAR(56) NOT NULL,
    holder_address VARCHAR(56) NOT NULL,
    share_balance NUMERIC(36, 7) NOT NULL DEFAULT 0,
    locked_until BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (vault_id, holder_address)
);
