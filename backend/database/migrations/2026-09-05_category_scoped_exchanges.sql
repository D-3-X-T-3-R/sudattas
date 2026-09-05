-- Category-scoped exchanges: same product, different size/colour, exact same price — a swap,
-- distinct from the refund-only ReturnRequests pipeline. Admin opts individual categories in via
-- exchange_eligible; a new ExchangeRequests table tracks the request through to the replacement
-- order (created via the existing place_order_admin path once the original item is received).

ALTER TABLE ProductCategories
    ADD COLUMN exchange_eligible TINYINT(1) NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS ExchangeRequests (
    exchange_id BIGINT NOT NULL AUTO_INCREMENT,
    order_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    order_detail_id BIGINT NOT NULL,
    desired_variant_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'requested',
    reason VARCHAR(512) NOT NULL,
    replacement_order_id BIGINT NULL DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received_at TIMESTAMP NULL DEFAULT NULL,
    PRIMARY KEY (exchange_id),
    KEY idx_exchange_requests_order (order_id),
    KEY idx_exchange_requests_user (user_id),
    KEY idx_exchange_requests_status (status),
    KEY idx_exchange_requests_order_detail (order_detail_id),
    KEY idx_exchange_requests_replacement_order (replacement_order_id),
    CONSTRAINT fk_exchange_requests_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID),
    CONSTRAINT fk_exchange_requests_user
        FOREIGN KEY (user_id) REFERENCES Users (UserID),
    CONSTRAINT fk_exchange_requests_order_detail
        FOREIGN KEY (order_detail_id) REFERENCES OrderDetails (OrderDetailID),
    CONSTRAINT fk_exchange_requests_desired_variant
        FOREIGN KEY (desired_variant_id) REFERENCES ProductVariants (VariantID),
    CONSTRAINT fk_exchange_requests_replacement_order
        FOREIGN KEY (replacement_order_id) REFERENCES Orders (OrderID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
