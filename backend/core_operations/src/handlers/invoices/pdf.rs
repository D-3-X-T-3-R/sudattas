use super::InvoiceDocumentSnapshot;

fn sanitize_pdf_text(input: &str) -> String {
    input
        .chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect()
}

fn escape_pdf_text(input: &str) -> String {
    sanitize_pdf_text(input)
        .replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
}

fn build_content_stream(lines: &[String]) -> String {
    let mut stream = String::new();
    stream.push_str("BT\n");
    stream.push_str("/F1 11 Tf\n");
    stream.push_str("14 TL\n");
    stream.push_str("50 780 Td\n");
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            stream.push_str("T*\n");
        }
        stream.push('(');
        stream.push_str(&escape_pdf_text(line));
        stream.push_str(") Tj\n");
    }
    stream.push_str("ET\n");
    stream
}

fn build_lines(snapshot: &InvoiceDocumentSnapshot) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push("Sudatta's Tax Invoice".to_string());
    lines.push(String::new());
    lines.push(format!("Invoice Number: {}", snapshot.invoice_number));
    lines.push(format!("Order ID: {}", snapshot.order_id));
    lines.push(format!(
        "Generated At (UTC): {}",
        snapshot.generated_at_rfc3339
    ));
    lines.push(String::new());
    lines.push(format!("Customer Name: {}", snapshot.customer_name));
    lines.push(format!("Customer Email: {}", snapshot.customer_email));
    lines.push("Shipping Address:".to_string());
    for part in snapshot.shipping_address_snapshot.lines() {
        lines.push(format!("  {}", part));
    }
    lines.push(String::new());
    lines.push("Line Items:".to_string());
    for line in &snapshot.lines {
        lines.push(format!(
            "  - {} | Qty {} | Unit {} | Line {}",
            line.title, line.quantity, line.unit_price_formatted, line.line_total_formatted
        ));
    }
    lines.push(String::new());
    lines.push(format!("Item Total: {}", snapshot.item_total_formatted));
    lines.push(format!("Discount: {}", snapshot.discount_formatted));
    lines.push(format!("Shipping: {}", snapshot.shipping_formatted));
    lines.push(format!("Grand Total: {}", snapshot.grand_total_formatted));
    lines.push(format!("Payment Mode: {}", snapshot.payment_mode));
    lines.push(format!("Payment Status: {}", snapshot.payment_status));
    lines
}

pub fn render_invoice_pdf(snapshot: &InvoiceDocumentSnapshot) -> Vec<u8> {
    let all_lines = build_lines(snapshot);
    let lines_per_page = 45usize;
    let pages = all_lines
        .chunks(lines_per_page)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let page_count = pages.len().max(1);
    let font_obj_id = 3 + (page_count as i32) * 2;

    let mut objects: Vec<String> = Vec::new();
    objects.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());

    let mut kids = String::new();
    for idx in 0..page_count {
        let page_obj_id = 3 + (idx as i32) * 2;
        if !kids.is_empty() {
            kids.push(' ');
        }
        kids.push_str(&format!("{page_obj_id} 0 R"));
    }
    objects.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids, page_count
    ));

    for idx in 0..page_count {
        let page_obj_id = 3 + (idx as i32) * 2;
        let content_obj_id = page_obj_id + 1;
        let page_lines = pages
            .get(idx)
            .cloned()
            .unwrap_or_else(|| vec!["Sudatta's Tax Invoice".to_string()]);
        let stream = build_content_stream(&page_lines);

        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents {} 0 R /Resources << /Font << /F1 {} 0 R >> >> >>",
            content_obj_id, font_obj_id
        ));
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            stream.len(),
            stream
        ));
    }

    objects.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string());

    let mut pdf = String::from("%PDF-1.4\n");
    let mut offsets: Vec<usize> = vec![0];
    for (idx, object_body) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", idx + 1, object_body));
    }

    let xref_start = pdf.len();
    pdf.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
    pdf.push_str("0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.push_str(&format!("{:010} 00000 n \n", offset));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        objects.len() + 1,
        xref_start
    ));

    pdf.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::invoices::{InvoiceDocumentLineSnapshot, InvoiceDocumentSnapshot};

    #[test]
    fn render_invoice_pdf_produces_pdf_bytes() {
        let snapshot = InvoiceDocumentSnapshot {
            invoice_number: "INV-20260426-000123".to_string(),
            order_id: 123,
            customer_name: "Test User".to_string(),
            customer_email: "test@example.com".to_string(),
            shipping_address_snapshot: "Line 1\nBengaluru".to_string(),
            generated_at_rfc3339: "2026-04-26T12:00:00Z".to_string(),
            lines: vec![InvoiceDocumentLineSnapshot {
                title: "Silk Saree".to_string(),
                quantity: 1,
                unit_price_minor: 120_000,
                unit_price_formatted: "INR 1200.00".to_string(),
                line_total_minor: 120_000,
                line_total_formatted: "INR 1200.00".to_string(),
            }],
            item_total_minor: 120_000,
            discount_minor: 10_000,
            shipping_minor: 0,
            grand_total_minor: 110_000,
            item_total_formatted: "INR 1200.00".to_string(),
            discount_formatted: "-INR 100.00".to_string(),
            shipping_formatted: "INR 0.00".to_string(),
            grand_total_formatted: "INR 1100.00".to_string(),
            payment_mode: "prepaid".to_string(),
            payment_status: "captured".to_string(),
        };

        let bytes = render_invoice_pdf(&snapshot);
        assert!(!bytes.is_empty());
        let rendered = String::from_utf8_lossy(&bytes);
        assert!(rendered.starts_with("%PDF-1.4"));
        assert!(rendered.contains("INV-20260426-000123"));
    }
}
