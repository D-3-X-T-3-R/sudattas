-- Apply BEFORE deploying the commerce transaction patch. Files only; not executed.
-- Additive: legacy IdempotencyKeys, invoices, ledger and webhook payloads are retained.
CREATE TABLE IF NOT EXISTS CheckoutAttemptsV2 (
    customer_id BIGINT NOT NULL,
    operation VARCHAR(32) NOT NULL,
    request_key VARBINARY(191) NOT NULL,
    fingerprint CHAR(64) NOT NULL,
    fingerprint_version INT NOT NULL DEFAULT 2,
    request_json JSON NOT NULL,
    token CHAR(36) NOT NULL,
    state VARCHAR(32) NOT NULL,
    receipt VARCHAR(40) NOT NULL,
    gateway_order_id VARCHAR(100) NULL,
    amount_paise BIGINT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'INR',
    order_id BIGINT NULL,
    last_error VARCHAR(255) NULL,
    expires_at DATETIME(6) NOT NULL,
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY(customer_id, operation, request_key),
    KEY checkout_review(state, updated_at)
) ENGINE=InnoDB;

-- Unique by business effect, not by provider event ID. Existing captured orders
-- are guarded by their locked captured state; historical ledger repair is separate.
CREATE TABLE IF NOT EXISTS CommerceEffects (
    order_id BIGINT NOT NULL,
    effect VARCHAR(64) NOT NULL,
    provider_id VARCHAR(191) NOT NULL,
    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    PRIMARY KEY(order_id, effect),
    UNIQUE KEY commerce_provider_effect(effect, provider_id)
) ENGINE=InnoDB;

-- No backfill: old events lack verifiable provenance and cannot be trusted by replay.
CREATE TABLE IF NOT EXISTS WebhookVerificationV2 (
    event_id BIGINT NOT NULL PRIMARY KEY,
    raw_body LONGTEXT NOT NULL,
    body_sha256 CHAR(64) NOT NULL,
    verified_at DATETIME(6) NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    exhausted BOOLEAN NOT NULL DEFAULT FALSE,
    last_error VARCHAR(255) NULL,
    KEY webhook_retry(exhausted, next_attempt_at)
) ENGINE=InnoDB;

-- Refund association is immutable after first authenticated observation. Existing
-- Refunds are checked against their order/amount/currency before adopting them.
CREATE TABLE IF NOT EXISTS RefundEvidenceV2 (
    gateway_refund_id VARCHAR(191) NOT NULL PRIMARY KEY,
    payment_id VARCHAR(191) NOT NULL,
    order_id BIGINT NOT NULL,
    amount_paise BIGINT NOT NULL,
    currency CHAR(3) NOT NULL,
    provider_status VARCHAR(32) NOT NULL,
    needs_review BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS PaymentReconciliationV2 (
    intent_id BIGINT NOT NULL PRIMARY KEY,
    attempts INT NOT NULL DEFAULT 0,
    next_attempt_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    last_error VARCHAR(255) NULL
) ENGINE=InnoDB;

CREATE TABLE IF NOT EXISTS RefundSubmissionsV2 (
    attempt_id BIGINT NOT NULL PRIMARY KEY,
    state VARCHAR(32) NOT NULL,
    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)
) ENGINE=InnoDB;
-- Historical ambiguous submissions must not become fresh POSTs after deployment.
INSERT IGNORE INTO RefundSubmissionsV2(attempt_id,state)
SELECT attempt_id,'legacy_review' FROM RefundAttempts
WHERE gateway_refund_id IS NULL AND status NOT IN ('processed','failed');
