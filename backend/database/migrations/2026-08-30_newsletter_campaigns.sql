-- One row per admin "send newsletter to all subscribers" action. No draft/scheduled state:
-- a row is written once the send loop finishes, recording final subject/body/CTA plus
-- recipient/success/failure counts. Immutable history, not an editable document.

CREATE TABLE IF NOT EXISTS NewsletterCampaigns (
    campaign_id BIGINT NOT NULL AUTO_INCREMENT,
    subject VARCHAR(255) NOT NULL,
    body_text MEDIUMTEXT NOT NULL,
    cta_label VARCHAR(120) NULL,
    cta_url VARCHAR(500) NULL,
    recipient_count INT NOT NULL DEFAULT 0,
    success_count INT NOT NULL DEFAULT 0,
    failure_count INT NOT NULL DEFAULT 0,
    sent_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (campaign_id),
    KEY idx_newsletter_campaigns_sent_at (sent_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
