-- Full-order cancellation / refund saga: durable settlement state, refund attempts audit, inventory restore guard.

CREATE TABLE IF NOT EXISTS OrderInventoryRestores (
    order_id BIGINT NOT NULL,
    restored_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (order_id),
    CONSTRAINT fk_order_inventory_restores_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE IF NOT EXISTS RefundAttempts (
    attempt_id BIGINT NOT NULL AUTO_INCREMENT,
    order_id BIGINT NOT NULL,
    payment_intent_id BIGINT NULL,
    razorpay_payment_id VARCHAR(191) NULL,
    amount_requested_paise BIGINT NOT NULL,
    amount_sent_to_gateway_paise BIGINT NOT NULL,
    gateway_refund_id VARCHAR(191) NULL,
    status VARCHAR(32) NOT NULL,
    provider_error TEXT NULL,
    idempotency_key VARCHAR(191) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (attempt_id),
    KEY idx_refund_attempts_idempotency (idempotency_key),
    KEY idx_refund_attempts_order (order_id),
    CONSTRAINT fk_refund_attempts_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

SET @refund_settlement_status_exists := (
    SELECT COUNT(*)
    FROM information_schema.columns
    WHERE table_schema = DATABASE()
      AND table_name = 'Orders'
      AND column_name = 'refund_settlement_status'
);
SET @refund_settlement_status_sql := IF(
    @refund_settlement_status_exists = 0,
    'ALTER TABLE Orders ADD COLUMN refund_settlement_status VARCHAR(32) NULL DEFAULT NULL COMMENT ''refund_pending|refund_processed|refund_failed|refund_not_applicable''',
    'SELECT 1'
);
PREPARE refund_settlement_status_stmt FROM @refund_settlement_status_sql;
EXECUTE refund_settlement_status_stmt;
DEALLOCATE PREPARE refund_settlement_status_stmt;
