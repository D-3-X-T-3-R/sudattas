-- Generic admin-configurable settings store (key/value), so runtime tuning knobs like the
-- abandoned-cart worker's schedule no longer require an env var change + container restart.
-- Extensible for future settings without a new table per feature.

CREATE TABLE IF NOT EXISTS AppSettings (
    setting_key VARCHAR(100) NOT NULL,
    setting_value VARCHAR(255) NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (setting_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
