//! Shared branded HTML shell for every outbound email — transactional order emails and
//! newsletter campaigns alike — so what a customer actually sees is the site's real visual
//! identity (deep green + gold, Playfair Display headline) instead of a bare unstyled
//! `<body>`. Previously only `newsletter_mail.rs` had this treatment; `order_mail.rs`'s
//! order-lifecycle emails used plain `<p>`/`<ul>` with no styling at all.
//!
//! Table-based markup with inline styles throughout — email clients (Outlook especially)
//! don't reliably support external/embedded CSS.

pub fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

pub fn store_display_name() -> String {
    std::env::var("STORE_DISPLAY_NAME").unwrap_or_else(|_| "Sudatta's".to_string())
}

pub fn storefront_url() -> String {
    std::env::var("STOREFRONT_URL").unwrap_or_else(|_| "https://sudattas.com".to_string())
}

/// Same env var and default the PDF invoice's "Support:" line already uses (see
/// `handlers/invoices/pdf.rs`) — kept in sync rather than inventing a second contact address.
pub fn support_email() -> String {
    std::env::var("SUPPORT_EMAIL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "sudattasdesignerboutique@gmail.com".to_string())
}

/// One paragraph of body copy, styled consistently wherever it appears.
pub fn paragraph_html(text_escaped_html: &str) -> String {
    format!(
        "<p style=\"margin:0 0 18px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:15px;line-height:1.7;color:#2D2A26;\">{text_escaped_html}</p>"
    )
}

/// The single gold pill CTA button used across every email type.
pub fn cta_button_html(label: &str, url: &str) -> String {
    format!(
        "<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" style=\"margin:8px 0 4px;\"><tr><td style=\"border-radius:999px;background-color:#C9A646;\">\
        <a href=\"{}\" style=\"display:inline-block;padding:14px 34px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:14px;font-weight:600;color:#0F3D2E;text-decoration:none;border-radius:999px;\">{}</a>\
        </td></tr></table>",
        html_escape(url.trim()),
        html_escape(label.trim())
    )
}

/// Wrap already-built inner HTML (`body_html` — headline, paragraphs, tables, buttons; caller
/// is responsible for escaping any dynamic text within it) in the shared header/body/footer
/// shell. `footer_extra_html`, if given, is appended inside the footer below the standard
/// "you're receiving this" line (e.g. a newsletter's unsubscribe link) — transactional order
/// emails pass `None` and get a plain contact-us footer instead, since they aren't marketing
/// and don't need (or legally require) an unsubscribe option.
pub fn render_branded_email(
    subject: &str,
    body_html: &str,
    footer_extra_html: Option<&str>,
) -> String {
    let store = store_display_name();
    let site_url = storefront_url();
    let subject_escaped = html_escape(subject.trim());
    let site_url_display = html_escape(
        site_url
            .trim_start_matches("https://")
            .trim_start_matches("http://"),
    );

    let footer_line = match footer_extra_html {
        Some(extra) => format!(
            "You're receiving this because you subscribed at <a href=\"{}\" style=\"color:#6B6257;\">{}</a>.<br>{}",
            html_escape(&site_url),
            site_url_display,
            extra
        ),
        None => format!(
            "This is an automated message — please don't reply to this address. Questions? Contact us at <a href=\"mailto:{email}\" style=\"color:#6B6257;\">{email}</a> or visit <a href=\"{url}\" style=\"color:#6B6257;\">{url_display}</a>.",
            email = html_escape(&support_email()),
            url = html_escape(&site_url),
            url_display = site_url_display
        ),
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{subject_escaped}</title>
</head>
<body style="margin:0;padding:0;background-color:#F6F3EA;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:#F6F3EA;">
<tr><td align="center" style="padding:32px 16px;">
<table role="presentation" width="600" cellpadding="0" cellspacing="0" style="max-width:600px;width:100%;background-color:#FBF9F4;border-radius:14px;overflow:hidden;">
<tr><td style="background-color:#0F3D2E;padding:28px 32px;text-align:center;">
<div style="font-family:Georgia,'Times New Roman',serif;font-size:14px;letter-spacing:4px;color:#E7CF82;text-transform:uppercase;">{store_escaped}</div>
</td></tr>
<tr><td style="padding:40px 32px 8px;">
{body_html}
</td></tr>
<tr><td style="padding:24px 32px 40px;">
<hr style="border:none;border-top:1px solid #DED7C8;margin:0 0 20px;">
<p style="margin:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Helvetica,Arial,sans-serif;font-size:12px;line-height:1.6;color:#6B6257;">
{footer_line}
</p>
</td></tr>
</table>
</td></tr>
</table>
</body>
</html>"#,
        subject_escaped = subject_escaped,
        store_escaped = html_escape(&store),
        body_html = body_html,
        footer_line = footer_line,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_special_characters() {
        assert_eq!(html_escape("Sarees & More <3"), "Sarees &amp; More &lt;3");
    }

    #[test]
    fn render_includes_subject_store_and_body() {
        let html = render_branded_email("Order received", "<p>hi</p>", None);
        assert!(html.contains("Order received"));
        assert!(html.contains("<p>hi</p>"));
        assert!(html.contains("don't reply to this address"));
        assert!(html.contains(&support_email()));
    }

    #[test]
    fn footer_extra_html_is_appended_when_given() {
        let html = render_branded_email(
            "S",
            "<p>b</p>",
            Some("<a href=\"https://x/u\">Unsubscribe</a>"),
        );
        assert!(html.contains("Unsubscribe"));
        assert!(!html.contains("don't reply to this address"));
    }

    #[test]
    fn cta_button_contains_label_and_url() {
        let html = cta_button_html("Shop now", "https://sudattas.com/collections");
        assert!(html.contains("Shop now"));
        assert!(html.contains("https://sudattas.com/collections"));
    }
}
