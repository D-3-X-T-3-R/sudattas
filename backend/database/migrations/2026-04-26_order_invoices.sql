-- Immutable order invoices (PDF) with secure storage metadata.

CREATE TABLE IF NOT EXISTS Invoices (
    invoice_id BIGINT NOT NULL AUTO_INCREMENT,
    invoice_number VARCHAR(64) NOT NULL,
    order_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    generated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    storage_path VARCHAR(255) NOT NULL,
    pdf_blob LONGBLOB NOT NULL,
    snapshot_json JSON NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (invoice_id),
    UNIQUE KEY uq_invoices_invoice_number (invoice_number),
    UNIQUE KEY uq_invoices_order_id (order_id),
    KEY idx_invoices_user_id (user_id),
    KEY idx_invoices_generated_at (generated_at),
    CONSTRAINT fk_invoices_order
        FOREIGN KEY (order_id) REFERENCES Orders (OrderID)
        ON DELETE CASCADE,
    CONSTRAINT fk_invoices_user
        FOREIGN KEY (user_id) REFERENCES Users (UserID)
        ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

ALTER TABLE Orders
    ADD COLUMN invoice_id BIGINT NULL,
    ADD COLUMN invoice_number VARCHAR(64) NULL,
    ADD COLUMN invoice_generated_at TIMESTAMP NULL DEFAULT NULL,
    ADD COLUMN invoice_storage_path VARCHAR(255) NULL;

ALTER TABLE Orders
    ADD UNIQUE KEY uq_orders_invoice_id (invoice_id),
    ADD UNIQUE KEY uq_orders_invoice_number (invoice_number);

ALTER TABLE Orders
    ADD CONSTRAINT fk_orders_invoice
        FOREIGN KEY (invoice_id) REFERENCES Invoices (invoice_id)
        ON DELETE SET NULL;
