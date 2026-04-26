-- Prepaid post-delivery returns (partial item support) with durable refund linkage.

CREATE TABLE IF NOT EXISTS ReturnRequests (
    return_id BIGINT NOT NULL AUTO_INCREMENT,
    order_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'requested',
    reason VARCHAR(512) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    received_at TIMESTAMP NULL DEFAULT NULL,
    refund_attempt_id BIGINT NULL,
    PRIMARY KEY (return_id),
    KEY idx_return_requests_order (order_id),
    KEY idx_return_requests_user (user_id),
    KEY idx_return_requests_status (status),
    KEY idx_return_requests_refund_attempt (refund_attempt_id),
    CONSTRAINT fk_return_requests_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID),
    CONSTRAINT fk_return_requests_user
        FOREIGN KEY (user_id) REFERENCES Users (UserID),
    CONSTRAINT fk_return_requests_refund_attempt
        FOREIGN KEY (refund_attempt_id) REFERENCES RefundAttempts (attempt_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE IF NOT EXISTS ReturnRequestItems (
    return_id BIGINT NOT NULL,
    order_detail_id BIGINT NOT NULL,
    quantity BIGINT NOT NULL,
    refund_amount_minor BIGINT NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'requested',
    PRIMARY KEY (return_id, order_detail_id),
    KEY idx_return_items_order_detail (order_detail_id),
    KEY idx_return_items_status (status),
    CONSTRAINT fk_return_items_return
        FOREIGN KEY (return_id) REFERENCES ReturnRequests (return_id)
        ON DELETE CASCADE,
    CONSTRAINT fk_return_items_order_detail
        FOREIGN KEY (order_detail_id) REFERENCES OrderDetails (OrderDetailID)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
