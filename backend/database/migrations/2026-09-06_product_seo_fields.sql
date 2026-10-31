-- Product SEO fields: admin-editable meta title/description, distinct from the storefront's
-- product name/description which are copy, not tuned for search snippets. Nullable — when unset,
-- the storefront falls back to its existing generated title/description exactly as before.

ALTER TABLE Products
    ADD COLUMN meta_title VARCHAR(70) NULL DEFAULT NULL,
    ADD COLUMN meta_description VARCHAR(160) NULL DEFAULT NULL;
