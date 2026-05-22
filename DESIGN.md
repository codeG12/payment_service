## payment_service
I chose Rust as a developing language since I'm familiar and quite a 2 year experience with it. 

For web framework, I chose axum since it has the tokio runtime and it ensures unparalleled
concurrency performance allowing the server to handle thoushands of request with low latency and for postgres migration using sqlx migration seems optimal for given time constraint.
and for accessing data in postgres chose the sqlx libray since it's compile time verification performance.

## Database Model
```
CREATE TABLE businesses (
    id               TEXT        PRIMARY KEY,       -- ULID --since it's easy to index and unique
    name             TEXT        NOT NULL,
    api_key_hash     TEXT        NOT NULL UNIQUE,       -- argon2 hash of the raw key
    api_key_prefix   TEXT        NOT NULL,              -- first 8 chars, shown in UI
    is_active        BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE customers (
    id           TEXT        PRIMARY KEY,
    business_id  TEXT        NOT NULL REFERENCES businesses(id),
    name         TEXT        NOT NULL,
    email        TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (business_id, email)
);

CREATE TABLE invoices (
    id               TEXT        PRIMARY KEY,
    business_id      TEXT        NOT NULL REFERENCES businesses(id),
    customer_id      TEXT        NOT NULL REFERENCES customers(id),
    state            TEXT        NOT NULL DEFAULT 'draft',
    currency         CHAR(3)     NOT NULL DEFAULT 'USD',
    total_cents      BIGINT      NOT NULL,              -- server-computed, never from client
    due_date         TIMESTAMPTZ NOT NULL,
    idempotency_key  TEXT,                              -- optional, for creation idempotency
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (total_cents >= 0),
    CHECK (state IN ('draft','open','paid','void','uncollectible'))
);

CREATE TABLE line_items (
    id               TEXT    PRIMARY KEY,
    invoice_id       TEXT    NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description      TEXT    NOT NULL,
    quantity         INT     NOT NULL CHECK (quantity > 0),
    unit_amount_cents BIGINT NOT NULL CHECK (unit_amount_cents >= 0),
    amount_cents     BIGINT  NOT NULL,                 -- quantity * unit_amount_cents, computed on insert
    position         INT     NOT NULL DEFAULT 0        -- display order
);

CREATE TABLE payment_attempts (
    id               TEXT        PRIMARY KEY,
    invoice_id       TEXT        NOT NULL REFERENCES invoices(id),
    idempotency_key  TEXT        NOT NULL UNIQUE,      -- enforces idempotency
    card_token       TEXT        NOT NULL,
    amount_cents     BIGINT      NOT NULL,
    currency         CHAR(3)     NOT NULL DEFAULT 'USD',
    state            TEXT        NOT NULL DEFAULT 'pending',
    psp_reference_id TEXT,                             -- mock PSP transaction ID
    error_message    TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (state IN ('pending','succeeded','failed'))
);

CREATE TABLE webhook_deliveries (
    id                  TEXT        PRIMARY KEY,
    webhook_endpoint_id TEXT        NOT NULL REFERENCES webhook_endpoints(id),
    event_type          TEXT        NOT NULL,          -- 'invoice.created', 'invoice.paid', 'invoice.payment_failed'
    payload             JSONB       NOT NULL,
    state               TEXT        NOT NULL DEFAULT 'pending',
    attempt_count       INT         NOT NULL DEFAULT 0,
    next_retry_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_http_status    INT,
    last_error          TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (state IN ('pending','delivered','failed'))
);
CREATE INDEX idx_webhook_deliveries_retry ON webhook_deliveries(state, next_retry_at)
    WHERE state = 'pending';
```
