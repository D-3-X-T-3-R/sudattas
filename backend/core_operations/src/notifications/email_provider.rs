//! Send transactional email: Resend HTTP API or log-only (default).

use serde::Serialize;
use tonic::Status;
use tracing::info;

const RESEND_API: &str = "https://api.resend.com/emails";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailProviderKind {
    /// Log full message with tracing (default; no external calls).
    Log,
    /// POST to Resend (`RESEND_API_KEY`, `EMAIL_FROM` required).
    Resend,
}

#[derive(Debug, Clone)]
pub struct EmailAttachment {
    pub filename: String,
    pub content_base64: String,
    pub mime_type: String,
}

pub fn email_provider_from_env() -> EmailProviderKind {
    match std::env::var("EMAIL_PROVIDER")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "resend" => EmailProviderKind::Resend,
        _ => EmailProviderKind::Log,
    }
}

#[derive(Serialize)]
struct ResendBody<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    text: &'a str,
    html: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<ResendAttachment<'a>>,
}

#[derive(Serialize)]
struct ResendAttachment<'a> {
    filename: &'a str,
    content: &'a str,
}

/// Send one email. On `Log` mode, logs and returns Ok. On `Resend`, POST to API.
pub async fn send_transactional_email(
    to: &str,
    subject: &str,
    text_body: &str,
    html_body: &str,
) -> Result<(), Status> {
    send_transactional_email_with_attachments(to, subject, text_body, html_body, &[]).await
}

/// Send one email with optional attachments.
pub async fn send_transactional_email_with_attachments(
    to: &str,
    subject: &str,
    text_body: &str,
    html_body: &str,
    attachments: &[EmailAttachment],
) -> Result<(), Status> {
    let to = to.trim();
    if to.is_empty() {
        return Err(Status::invalid_argument("recipient email is empty"));
    }
    if attachments.iter().any(|a| a.filename.trim().is_empty()) {
        return Err(Status::invalid_argument(
            "attachment filename cannot be empty",
        ));
    }
    if attachments
        .iter()
        .any(|a| a.content_base64.trim().is_empty())
    {
        return Err(Status::invalid_argument(
            "attachment content cannot be empty",
        ));
    }

    match email_provider_from_env() {
        EmailProviderKind::Log => {
            info!(
                to = %to,
                subject = %subject,
                text_len = text_body.len(),
                html_len = html_body.len(),
                attachment_count = attachments.len(),
                "transactional email (EMAIL_PROVIDER=log - set EMAIL_PROVIDER=resend + RESEND_API_KEY to send)"
            );
            for attachment in attachments {
                info!(
                    file_name = %attachment.filename,
                    mime_type = %attachment.mime_type,
                    base64_len = attachment.content_base64.len(),
                    "transactional email attachment"
                );
            }
            info!(body = %text_body, "transactional email plain-text body");
            Ok(())
        }
        EmailProviderKind::Resend => {
            let key = std::env::var("RESEND_API_KEY")
                .map_err(|_| Status::failed_precondition("RESEND_API_KEY not set"))?;
            let from = std::env::var("EMAIL_FROM")
                .map_err(|_| Status::failed_precondition("EMAIL_FROM not set"))?;

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| Status::internal(e.to_string()))?;

            let body = ResendBody {
                from: from.as_str(),
                to: vec![to],
                subject,
                text: text_body,
                html: html_body,
                attachments: attachments
                    .iter()
                    .map(|attachment| ResendAttachment {
                        filename: attachment.filename.as_str(),
                        content: attachment.content_base64.as_str(),
                    })
                    .collect(),
            };

            let res = client
                .post(RESEND_API)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| Status::internal(format!("Resend request failed: {e}")))?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(Status::internal(format!(
                    "Resend error HTTP {}: {}",
                    status, err_text
                )));
            }
            info!(
                to = %to,
                subject = %subject,
                attachment_count = attachments.len(),
                "transactional email sent via Resend"
            );
            Ok(())
        }
    }
}
