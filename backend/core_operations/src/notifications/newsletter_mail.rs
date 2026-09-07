//! Newsletter campaign email content — the header/body/footer shell itself now lives in
//! `brand_email.rs`, shared with every transactional order email so the whole site's outbound
//! mail looks the same. This module only builds the campaign-specific inner content: the
//! subject headline, paragraphs, and CTA button, plus the unsubscribe footer link.

use super::brand_email::{cta_button_html, html_escape, paragraph_html, render_branded_email};

/// Body text -> HTML paragraphs. Splits on blank lines (paragraph breaks); a single newline
/// within a paragraph becomes `<br>`. Escapes everything — the text itself is admin-authored
/// and trusted, but stray `&`/`<` (an ampersand in "Sarees & More", say) would otherwise
/// silently break the rendered email.
fn body_paragraphs_html(body_text: &str) -> String {
    body_text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| paragraph_html(&html_escape(p).replace('\n', "<br>")))
        .collect::<Vec<_>>()
        .join("")
}

fn body_paragraphs_text(body_text: &str) -> String {
    body_text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Render the full campaign email for one recipient. `unsubscribe_url` is per-recipient (it
/// embeds that subscriber's id + signed token), so this must be called once per recipient
/// rather than rendered once and reused.
pub fn build_newsletter_campaign_email(
    subject: &str,
    body_text: &str,
    cta_label: Option<&str>,
    cta_url: Option<&str>,
    unsubscribe_url: &str,
) -> (String, String) {
    let subject_escaped = html_escape(subject.trim());

    let headline_html = format!(
        "<h1 style=\"margin:0 0 20px;font-family:Georgia,'Times New Roman',serif;font-size:27px;line-height:1.3;color:#2D2A26;font-weight:600;\">{subject_escaped}</h1>"
    );

    let cta_html = match (cta_label, cta_url) {
        (Some(label), Some(url)) if !label.trim().is_empty() && !url.trim().is_empty() => {
            cta_button_html(label, url)
        }
        _ => String::new(),
    };

    let body_html = format!(
        "{headline_html}{paragraphs}{cta_html}",
        headline_html = headline_html,
        paragraphs = body_paragraphs_html(body_text),
        cta_html = cta_html,
    );

    let unsubscribe_link = format!(
        "<a href=\"{}\" style=\"color:#6B6257;text-decoration:underline;\">Unsubscribe</a>",
        html_escape(unsubscribe_url)
    );
    let html = render_branded_email(subject, &body_html, Some(&unsubscribe_link));

    let mut text = String::new();
    text.push_str(subject.trim());
    text.push_str("\n\n");
    text.push_str(&body_paragraphs_text(body_text));
    if let (Some(label), Some(url)) = (cta_label, cta_url) {
        if !label.trim().is_empty() && !url.trim().is_empty() {
            text.push_str(&format!("\n\n{} — {}", label.trim(), url.trim()));
        }
    }
    text.push_str(&format!("\n\n---\nUnsubscribe: {}", unsubscribe_url));

    (text, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_ampersand_in_subject_and_body() {
        let (text, html) = build_newsletter_campaign_email(
            "Sarees & More",
            "New arrivals & festive picks.",
            None,
            None,
            "https://sudattas.com/newsletter/unsubscribe?id=1&token=abc",
        );
        assert!(html.contains("Sarees &amp; More"));
        assert!(html.contains("New arrivals &amp; festive picks."));
        // Plain-text version is unescaped.
        assert!(text.contains("Sarees & More"));
    }

    #[test]
    fn splits_body_into_paragraphs() {
        let (_, html) = build_newsletter_campaign_email(
            "Subject",
            "First paragraph.\n\nSecond paragraph.",
            None,
            None,
            "https://sudattas.com/u",
        );
        assert!(html.contains("First paragraph."));
        assert!(html.contains("Second paragraph."));
        // Body paragraphs use a distinct style from the footer's own `<p>` — count only those.
        assert_eq!(html.matches("margin:0 0 18px").count(), 2);
    }

    #[test]
    fn omits_cta_block_when_not_provided() {
        let (_, html) = build_newsletter_campaign_email("S", "Body.", None, None, "https://x/u");
        assert!(!html.contains("border-radius:999px;background-color:#C9A646"));
    }

    #[test]
    fn includes_cta_button_when_provided() {
        let (text, html) = build_newsletter_campaign_email(
            "S",
            "Body.",
            Some("Shop now"),
            Some("https://sudattas.com/collections"),
            "https://x/u",
        );
        assert!(html.contains("Shop now"));
        assert!(html.contains("https://sudattas.com/collections"));
        assert!(text.contains("Shop now"));
    }

    #[test]
    fn always_includes_unsubscribe_link() {
        let (text, html) = build_newsletter_campaign_email(
            "S",
            "Body.",
            None,
            None,
            "https://sudattas.com/newsletter/unsubscribe?id=7&token=xyz",
        );
        assert!(html.contains("Unsubscribe"));
        assert!(html.contains("id=7&amp;token=xyz"));
        assert!(text.contains("id=7&token=xyz"));
    }
}
