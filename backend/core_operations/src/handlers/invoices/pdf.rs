use super::InvoiceDocumentSnapshot;
use chrono::{DateTime, FixedOffset};
use genpdf::elements::{self, CellDecorator, LinearLayout, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Element as _, Margins, PaperSize, Position};
use tracing::warn;

const COLOR_PRIMARY: Color = Color::Rgb(0x0F, 0x3D, 0x3E);
const COLOR_ACCENT: Color = Color::Rgb(0xC6, 0xA7, 0x5E);
const COLOR_TEXT: Color = Color::Rgb(0x1A, 0x1A, 0x1A);
const COLOR_SECONDARY: Color = Color::Rgb(0x6B, 0x6B, 0x6B);
const COLOR_DIVIDER: Color = Color::Rgb(0xE5, 0xE5, 0xE5);
const MAX_EXPECTED_PDF_BYTES: usize = 300 * 1024;

const PLAYFAIR_REGULAR: &[u8] = include_bytes!("assets/fonts/PlayfairDisplay-Regular.ttf");
const INTER_REGULAR: &[u8] = include_bytes!("assets/fonts/Inter-Regular.ttf");

const IST_OFFSET_SECONDS: i32 = 5 * 3600 + 30 * 60;
const DEFAULT_SELLER_NAME: &str = "Sudatta's";
const DEFAULT_SUPPORT_EMAIL: &str = "support@sudattas.com";

#[derive(Clone)]
struct SellerInfo {
    name: String,
    address_lines: Vec<String>,
    email: String,
    phone: String,
}

#[derive(Clone, Copy)]
struct InvoiceStyles {
    heading_font: genpdf::fonts::FontFamily<genpdf::fonts::Font>,
}

impl InvoiceStyles {
    fn title(self) -> Style {
        Style::new()
            .with_font_family(self.heading_font)
            .with_font_size(24)
            .with_color(COLOR_PRIMARY)
    }

    fn header_tag(self) -> Style {
        Style::new()
            .with_font_size(11)
            .with_color(COLOR_ACCENT)
            .bold()
    }

    fn header_meta(self) -> Style {
        Style::new().with_font_size(10).with_color(COLOR_SECONDARY)
    }

    fn section_header(self) -> Style {
        Style::new()
            .with_font_size(11)
            .with_color(COLOR_PRIMARY)
            .bold()
    }

    fn label(self) -> Style {
        Style::new().with_font_size(9).with_color(COLOR_SECONDARY)
    }

    fn body(self) -> Style {
        Style::new().with_font_size(10).with_color(COLOR_TEXT)
    }

    fn table_header(self) -> Style {
        Style::new()
            .with_font_size(9)
            .with_color(COLOR_PRIMARY)
            .bold()
    }

    fn table_body(self) -> Style {
        Style::new().with_font_size(9).with_color(COLOR_TEXT)
    }

    fn summary_total(self) -> Style {
        Style::new()
            .with_font_size(12)
            .with_color(COLOR_PRIMARY)
            .bold()
    }

    fn footer(self) -> Style {
        Style::new().with_font_size(9).with_color(COLOR_SECONDARY)
    }
}

#[derive(Default)]
struct MinimalRowDivider {
    total_rows: usize,
}

impl CellDecorator for MinimalRowDivider {
    fn set_table_size(&mut self, _num_columns: usize, num_rows: usize) {
        self.total_rows = num_rows;
    }

    fn decorate_cell(
        &mut self,
        _column: usize,
        row: usize,
        _has_more: bool,
        area: genpdf::render::Area<'_>,
        style: Style,
    ) {
        let line_style = style.and(COLOR_DIVIDER);
        let size = area.size();
        if row == 0 {
            area.draw_line(
                vec![Position::new(0, 0), Position::new(size.width, 0)],
                line_style,
            );
        }
        area.draw_line(
            vec![
                Position::new(0, size.height),
                Position::new(size.width, size.height),
            ],
            line_style,
        );
        if row + 1 == self.total_rows {
            area.draw_line(
                vec![
                    Position::new(0, size.height),
                    Position::new(size.width, size.height),
                ],
                line_style,
            );
        }
    }
}

fn load_inter_family() -> Result<FontFamily<FontData>, genpdf::error::Error> {
    let regular = FontData::new(INTER_REGULAR.to_vec(), None)?;
    let bold = FontData::new(INTER_REGULAR.to_vec(), None)?;
    let italic = FontData::new(INTER_REGULAR.to_vec(), None)?;
    let bold_italic = FontData::new(INTER_REGULAR.to_vec(), None)?;
    Ok(FontFamily {
        regular,
        bold,
        italic,
        bold_italic,
    })
}

fn load_playfair_family() -> Result<FontFamily<FontData>, genpdf::error::Error> {
    let regular = FontData::new(PLAYFAIR_REGULAR.to_vec(), None)?;
    let bold = FontData::new(PLAYFAIR_REGULAR.to_vec(), None)?;
    let italic = FontData::new(PLAYFAIR_REGULAR.to_vec(), None)?;
    let bold_italic = FontData::new(PLAYFAIR_REGULAR.to_vec(), None)?;
    Ok(FontFamily {
        regular,
        bold,
        italic,
        bold_italic,
    })
}

fn seller_info() -> SellerInfo {
    let name = std::env::var("SELLER_BUSINESS_NAME")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("STORE_DISPLAY_NAME")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
        .unwrap_or_else(|| DEFAULT_SELLER_NAME.to_string());

    let mut address_lines = Vec::new();
    for key in [
        "SELLER_ADDRESS_LINE1",
        "SELLER_ADDRESS_LINE2",
        "SELLER_ADDRESS_LINE3",
    ] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                address_lines.push(trimmed.to_string());
            }
        }
    }
    if address_lines.is_empty() {
        address_lines.push("Online Retail, India".to_string());
    }

    let email = std::env::var("SUPPORT_EMAIL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_SUPPORT_EMAIL.to_string());

    let phone = std::env::var("SELLER_SUPPORT_PHONE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Not available".to_string());

    SellerInfo {
        name,
        address_lines,
        email,
        phone,
    }
}

fn format_invoice_datetime(raw: &str) -> String {
    let parsed = DateTime::parse_from_rfc3339(raw);
    let Some(ist) = FixedOffset::east_opt(IST_OFFSET_SECONDS) else {
        return raw.to_string();
    };
    parsed
        .map(|dt| {
            dt.with_timezone(&ist)
                .format("%d %b %Y, %I:%M %p IST")
                .to_string()
        })
        .unwrap_or_else(|_| raw.to_string())
}

fn format_inr(minor: i64) -> String {
    let sign = if minor < 0 { "-" } else { "" };
    let abs_minor = minor.unsigned_abs();
    let major = abs_minor / 100;
    let fraction = abs_minor % 100;
    format!("{sign}{}.{fraction:02}", format_inr_major(major))
}

fn format_inr_major(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 4);
    for (idx, ch) in digits.chars().enumerate() {
        if idx > 0 && (digits.len() - idx) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("₹{out}")
}

fn payment_mode_label(raw: &str) -> &'static str {
    if raw.eq_ignore_ascii_case("cod") {
        "Cash on Delivery"
    } else {
        "Prepaid"
    }
}

fn payment_status_label(raw_mode: &str, raw_status: &str) -> &'static str {
    if raw_mode.eq_ignore_ascii_case("cod") {
        "To be collected on delivery"
    } else if raw_status.eq_ignore_ascii_case("captured") || raw_status.eq_ignore_ascii_case("paid")
    {
        "Paid"
    } else {
        "Paid"
    }
}

fn split_shipping(snapshot: &InvoiceDocumentSnapshot) -> (Vec<String>, Option<String>) {
    let mut address_lines = Vec::new();
    let mut phone: Option<String> = None;
    for line in snapshot
        .shipping_address_snapshot
        .lines()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if phone.is_none() && looks_like_phone(line) {
            phone = Some(line.to_string());
        } else {
            address_lines.push(line.to_string());
        }
    }
    (address_lines, phone)
}

fn looks_like_phone(candidate: &str) -> bool {
    let digits = candidate.chars().filter(|ch| ch.is_ascii_digit()).count();
    digits >= 10
}

fn padded_text(text: impl Into<String>, style: Style, align: Alignment) -> impl genpdf::Element {
    Paragraph::new(text.into())
        .aligned(align)
        .styled(style)
        .padded(Margins::trbl(1.5, 1.0, 1.5, 1.0))
}

fn render_header(
    layout: &mut LinearLayout,
    snapshot: &InvoiceDocumentSnapshot,
    styles: InvoiceStyles,
    seller: &SellerInfo,
) -> Result<(), genpdf::error::Error> {
    let mut header = TableLayout::new(vec![3, 2]);

    let mut left = LinearLayout::vertical();
    left.push(Paragraph::new(seller.name.clone()).styled(styles.title()));

    let mut right = LinearLayout::vertical();
    right.push(
        Paragraph::new("TAX INVOICE")
            .aligned(Alignment::Right)
            .styled(styles.header_tag()),
    );
    right.push(
        Paragraph::new(format!("Invoice Number: {}", snapshot.invoice_number))
            .aligned(Alignment::Right)
            .styled(styles.header_meta()),
    );
    right.push(
        Paragraph::new(format!(
            "Invoice Date: {}",
            format_invoice_datetime(&snapshot.generated_at_rfc3339)
        ))
        .aligned(Alignment::Right)
        .styled(styles.header_meta()),
    );
    right.push(
        Paragraph::new(format!("Order ID: {}", snapshot.order_id))
            .aligned(Alignment::Right)
            .styled(styles.header_meta()),
    );

    header.row().element(left).element(right).push()?;
    layout.push(header);
    layout.push(elements::Break::new(1.3));
    layout.push(elements::Break::new(1.0));
    Ok(())
}

fn render_seller(layout: &mut LinearLayout, styles: InvoiceStyles, seller: &SellerInfo) {
    layout.push(Paragraph::new("Sold By").styled(styles.section_header()));
    layout.push(Paragraph::new(seller.name.clone()).styled(styles.body()));
    for line in &seller.address_lines {
        layout.push(Paragraph::new(line.clone()).styled(styles.body()));
    }
    layout.push(Paragraph::new(format!("Email: {}", seller.email)).styled(styles.body()));
    layout.push(Paragraph::new(format!("Phone: {}", seller.phone)).styled(styles.body()));
    layout.push(elements::Break::new(1.3));
}

fn render_customer(
    layout: &mut LinearLayout,
    snapshot: &InvoiceDocumentSnapshot,
    styles: InvoiceStyles,
) -> Result<(), genpdf::error::Error> {
    let mut customer_table = TableLayout::new(vec![1, 1]);
    let (ship_lines, ship_phone) = split_shipping(snapshot);

    let mut bill_to = LinearLayout::vertical();
    bill_to.push(Paragraph::new("Bill To").styled(styles.section_header()));
    bill_to.push(Paragraph::new(snapshot.customer_name.clone()).styled(styles.body()));
    bill_to.push(Paragraph::new(snapshot.customer_email.clone()).styled(styles.body()));

    let mut ship_to = LinearLayout::vertical();
    ship_to.push(Paragraph::new("Ship To").styled(styles.section_header()));
    for line in ship_lines {
        ship_to.push(Paragraph::new(line).styled(styles.body()));
    }
    if let Some(phone) = ship_phone {
        ship_to.push(Paragraph::new(format!("Phone: {}", phone)).styled(styles.body()));
    }

    customer_table
        .row()
        .element(bill_to.padded(Margins::trbl(0, 6.0, 0, 0)))
        .element(ship_to.padded(Margins::trbl(0, 0, 0, 6.0)))
        .push()?;
    layout.push(customer_table);
    layout.push(elements::Break::new(1.4));
    Ok(())
}

fn render_items_table(
    layout: &mut LinearLayout,
    snapshot: &InvoiceDocumentSnapshot,
    styles: InvoiceStyles,
) -> Result<(), genpdf::error::Error> {
    let mut table = TableLayout::new(vec![6, 1, 2, 2]);
    table.set_cell_decorator(MinimalRowDivider::default());

    table
        .row()
        .element(padded_text("Item", styles.table_header(), Alignment::Left))
        .element(padded_text("Qty", styles.table_header(), Alignment::Right))
        .element(padded_text(
            "Unit Price",
            styles.table_header(),
            Alignment::Right,
        ))
        .element(padded_text(
            "Total",
            styles.table_header(),
            Alignment::Right,
        ))
        .push()?;

    for line in &snapshot.lines {
        table
            .row()
            .element(padded_text(
                line.title.clone(),
                styles.table_body(),
                Alignment::Left,
            ))
            .element(padded_text(
                line.quantity.to_string(),
                styles.table_body(),
                Alignment::Right,
            ))
            .element(padded_text(
                format_inr(line.unit_price_minor),
                styles.table_body(),
                Alignment::Right,
            ))
            .element(padded_text(
                format_inr(line.line_total_minor),
                styles.table_body(),
                Alignment::Right,
            ))
            .push()?;
    }

    layout.push(table);
    layout.push(elements::Break::new(1.2));
    Ok(())
}

fn render_summary(
    layout: &mut LinearLayout,
    snapshot: &InvoiceDocumentSnapshot,
    styles: InvoiceStyles,
) -> Result<(), genpdf::error::Error> {
    let mut summary = TableLayout::new(vec![3, 2]);
    summary.set_cell_decorator(MinimalRowDivider::default());

    summary
        .row()
        .element(padded_text("Item Total", styles.body(), Alignment::Right))
        .element(padded_text(
            format_inr(snapshot.item_total_minor),
            styles.body(),
            Alignment::Right,
        ))
        .push()?;
    summary
        .row()
        .element(padded_text("Discount", styles.body(), Alignment::Right))
        .element(padded_text(
            format_inr(-snapshot.discount_minor),
            styles.body(),
            Alignment::Right,
        ))
        .push()?;
    summary
        .row()
        .element(padded_text("Shipping", styles.body(), Alignment::Right))
        .element(padded_text(
            format_inr(snapshot.shipping_minor),
            styles.body(),
            Alignment::Right,
        ))
        .push()?;
    summary
        .row()
        .element(padded_text(
            "Grand Total",
            styles.summary_total(),
            Alignment::Right,
        ))
        .element(padded_text(
            format_inr(snapshot.grand_total_minor),
            styles.summary_total(),
            Alignment::Right,
        ))
        .push()?;

    layout.push(summary.padded(Margins::trbl(0, 0, 0, 70.0)));
    layout.push(elements::Break::new(1.3));

    let mut payment = TableLayout::new(vec![3, 2]);
    payment
        .row()
        .element(padded_text("Payment Mode", styles.label(), Alignment::Left))
        .element(padded_text(
            payment_mode_label(&snapshot.payment_mode),
            styles.body(),
            Alignment::Left,
        ))
        .push()?;
    payment
        .row()
        .element(padded_text(
            "Payment Status",
            styles.label(),
            Alignment::Left,
        ))
        .element(padded_text(
            payment_status_label(&snapshot.payment_mode, &snapshot.payment_status),
            styles.body(),
            Alignment::Left,
        ))
        .push()?;
    layout.push(payment);
    layout.push(elements::Break::new(1.4));
    Ok(())
}

fn render_footer(layout: &mut LinearLayout, styles: InvoiceStyles, seller: &SellerInfo) {
    layout.push(
        Paragraph::new("This is a computer-generated invoice and does not require a signature.")
            .styled(styles.footer()),
    );
    layout.push(Paragraph::new(format!("Support: {}", seller.email)).styled(styles.footer()));
}

fn render_invoice_pdf_inner(
    snapshot: &InvoiceDocumentSnapshot,
) -> Result<Vec<u8>, genpdf::error::Error> {
    let default_family = load_inter_family()?;
    let heading_family_raw = load_playfair_family()?;

    let mut doc = genpdf::Document::new(default_family);
    let heading_family = doc.add_font_family(heading_family_raw);
    let styles = InvoiceStyles {
        heading_font: heading_family,
    };

    doc.set_title(format!("Invoice {}", snapshot.invoice_number));
    doc.set_font_size(10);
    doc.set_line_spacing(1.25);
    doc.set_paper_size(PaperSize::A4);
    doc.set_minimal_conformance();

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(20.0, 18.0, 20.0, 18.0));
    doc.set_page_decorator(decorator);

    let seller = seller_info();
    let mut layout = LinearLayout::vertical();
    render_header(&mut layout, snapshot, styles, &seller)?;
    render_seller(&mut layout, styles, &seller);
    render_customer(&mut layout, snapshot, styles)?;
    render_items_table(&mut layout, snapshot, styles)?;
    render_summary(&mut layout, snapshot, styles)?;
    render_footer(&mut layout, styles, &seller);
    doc.push(layout);

    let mut pdf_bytes = Vec::new();
    doc.render(&mut pdf_bytes)?;
    Ok(pdf_bytes)
}

fn render_failure_pdf(snapshot: &InvoiceDocumentSnapshot) -> Result<Vec<u8>, genpdf::error::Error> {
    let default_family = load_inter_family()?;
    let mut doc = genpdf::Document::new(default_family);
    doc.set_title(format!("Invoice {}", snapshot.invoice_number));
    doc.set_font_size(10);
    doc.set_paper_size(PaperSize::A4);
    doc.set_minimal_conformance();

    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(20.0, 18.0, 20.0, 18.0));
    doc.set_page_decorator(decorator);

    let mut layout = LinearLayout::vertical();
    layout.push(
        Paragraph::new("Tax Invoice").styled(
            Style::new()
                .with_font_size(18)
                .with_color(COLOR_PRIMARY)
                .bold(),
        ),
    );
    layout.push(elements::Break::new(0.8));
    layout.push(
        Paragraph::new(format!("Invoice Number: {}", snapshot.invoice_number))
            .styled(Style::new().with_color(COLOR_TEXT)),
    );
    layout.push(
        Paragraph::new(format!("Order ID: {}", snapshot.order_id))
            .styled(Style::new().with_color(COLOR_TEXT)),
    );
    layout.push(
        Paragraph::new(format!(
            "Grand Total: {}",
            format_inr(snapshot.grand_total_minor)
        ))
        .styled(Style::new().with_color(COLOR_TEXT).bold()),
    );
    layout.push(elements::Break::new(0.8));
    layout.push(
        Paragraph::new("Invoice rendering had a temporary formatting issue.")
            .styled(Style::new().with_color(COLOR_SECONDARY)),
    );
    doc.push(layout);

    let mut pdf_bytes = Vec::new();
    doc.render(&mut pdf_bytes)?;
    Ok(pdf_bytes)
}

pub fn render_invoice_pdf(snapshot: &InvoiceDocumentSnapshot) -> Vec<u8> {
    match render_invoice_pdf_inner(snapshot) {
        Ok(bytes) => {
            if bytes.len() > MAX_EXPECTED_PDF_BYTES {
                warn!(
                    invoice_number = %snapshot.invoice_number,
                    order_id = snapshot.order_id,
                    pdf_size_bytes = bytes.len(),
                    size_limit_bytes = MAX_EXPECTED_PDF_BYTES,
                    "premium invoice PDF size exceeds expected budget"
                );
            }
            bytes
        }
        Err(err) => {
            warn!(
                invoice_number = %snapshot.invoice_number,
                order_id = snapshot.order_id,
                error = %err,
                "premium invoice render failed; generating safe fallback invoice PDF"
            );
            render_failure_pdf(snapshot).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::invoices::{InvoiceDocumentLineSnapshot, InvoiceDocumentSnapshot};
    use pdf_extract::extract_text_from_mem;

    fn normalize_ws(input: &str) -> String {
        input.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn snapshot_with_lines(lines: Vec<InvoiceDocumentLineSnapshot>) -> InvoiceDocumentSnapshot {
        InvoiceDocumentSnapshot {
            invoice_number: "INV-20260426-000123".to_string(),
            order_id: 123,
            customer_name: "Test User".to_string(),
            customer_email: "test@example.com".to_string(),
            shipping_address_snapshot: "Invoice Test\n+919999999999\nLine 1\nBengaluru".to_string(),
            generated_at_rfc3339: "2026-04-26T12:00:00Z".to_string(),
            lines,
            item_total_minor: 170_000,
            discount_minor: 10_000,
            shipping_minor: 0,
            grand_total_minor: 160_000,
            item_total_formatted: "INR 1700.00".to_string(),
            discount_formatted: "-INR 100.00".to_string(),
            shipping_formatted: "INR 0.00".to_string(),
            grand_total_formatted: "INR 1600.00".to_string(),
            payment_mode: "prepaid".to_string(),
            payment_status: "captured".to_string(),
        }
    }

    #[test]
    fn render_invoice_pdf_produces_pdf_bytes() {
        let snapshot = snapshot_with_lines(vec![InvoiceDocumentLineSnapshot {
            title: "Silk Saree".to_string(),
            quantity: 1,
            unit_price_minor: 120_000,
            unit_price_formatted: "INR 1200.00".to_string(),
            line_total_minor: 120_000,
            line_total_formatted: "INR 1200.00".to_string(),
        }]);

        let bytes = render_invoice_pdf(&snapshot);
        assert!(!bytes.is_empty());
        let rendered = String::from_utf8_lossy(&bytes);
        assert!(rendered.starts_with("%PDF-"));
        assert!(rendered.contains("INV-20260426-000123"));
    }

    #[test]
    fn render_invoice_pdf_supports_multiple_items() {
        let snapshot = snapshot_with_lines(vec![
            InvoiceDocumentLineSnapshot {
                title: "Silk Saree".to_string(),
                quantity: 1,
                unit_price_minor: 120_000,
                unit_price_formatted: "INR 1200.00".to_string(),
                line_total_minor: 120_000,
                line_total_formatted: "INR 1200.00".to_string(),
            },
            InvoiceDocumentLineSnapshot {
                title: "Blouse".to_string(),
                quantity: 2,
                unit_price_minor: 25_000,
                unit_price_formatted: "INR 250.00".to_string(),
                line_total_minor: 50_000,
                line_total_formatted: "INR 500.00".to_string(),
            },
        ]);

        let bytes = render_invoice_pdf(&snapshot);
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 8_000);
    }

    #[test]
    fn render_invoice_pdf_stays_under_size_budget() {
        let snapshot = snapshot_with_lines(vec![
            InvoiceDocumentLineSnapshot {
                title: "Premium Silk Saree - Handwoven".to_string(),
                quantity: 1,
                unit_price_minor: 120_000,
                unit_price_formatted: "INR 1200.00".to_string(),
                line_total_minor: 120_000,
                line_total_formatted: "INR 1200.00".to_string(),
            },
            InvoiceDocumentLineSnapshot {
                title: "Blouse".to_string(),
                quantity: 2,
                unit_price_minor: 25_000,
                unit_price_formatted: "INR 250.00".to_string(),
                line_total_minor: 50_000,
                line_total_formatted: "INR 500.00".to_string(),
            },
            InvoiceDocumentLineSnapshot {
                title: "Petticoat".to_string(),
                quantity: 1,
                unit_price_minor: 12_500,
                unit_price_formatted: "INR 125.00".to_string(),
                line_total_minor: 12_500,
                line_total_formatted: "INR 125.00".to_string(),
            },
        ]);

        let bytes = render_invoice_pdf(&snapshot);
        eprintln!("invoice pdf size={} bytes", bytes.len());
        assert!(
            bytes.len() < MAX_EXPECTED_PDF_BYTES,
            "invoice PDF size {} exceeds limit {}",
            bytes.len(),
            MAX_EXPECTED_PDF_BYTES
        );
    }

    #[test]
    fn render_invoice_pdf_text_extraction_preserves_spaces() {
        let mut snapshot = snapshot_with_lines(vec![
            InvoiceDocumentLineSnapshot {
                title: "Premium Silk Saree - Handwoven".to_string(),
                quantity: 1,
                unit_price_minor: 120_000,
                unit_price_formatted: "INR 1200.00".to_string(),
                line_total_minor: 120_000,
                line_total_formatted: "INR 1200.00".to_string(),
            },
            InvoiceDocumentLineSnapshot {
                title: "Blouse".to_string(),
                quantity: 2,
                unit_price_minor: 25_000,
                unit_price_formatted: "INR 250.00".to_string(),
                line_total_minor: 50_000,
                line_total_formatted: "INR 500.00".to_string(),
            },
        ]);
        snapshot.payment_mode = "cod".to_string();
        snapshot.payment_status = "pending".to_string();

        let bytes = render_invoice_pdf(&snapshot);
        let text = extract_text_from_mem(&bytes).expect("extractable pdf text");
        let normalized = normalize_ws(&text);

        assert!(
            normalized.contains("TAX INVOICE"),
            "missing phrase TAX INVOICE in extracted text: {normalized}"
        );
        assert!(
            normalized.contains("Invoice Number"),
            "missing phrase Invoice Number in extracted text: {normalized}"
        );
        assert!(
            normalized.contains("Sold By"),
            "missing phrase Sold By in extracted text: {normalized}"
        );
        assert!(
            normalized.contains("Cash on Delivery"),
            "missing phrase Cash on Delivery in extracted text: {normalized}"
        );
        assert!(
            normalized.contains("To be collected on delivery"),
            "missing phrase To be collected on delivery in extracted text: {normalized}"
        );
    }
}
