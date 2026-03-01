//! P2 SEO: robots.txt and sitemap.xml (product URLs from gRPC).

use crate::resolvers::error::GqlError;
use crate::resolvers::utils::connect_grpc_client;
use proto::proto::core::GetSitemapProductUrlsRequest;
use proto::tonic::Request;

/// Static robots.txt body. Set SITEMAP_BASE_URL to point to full sitemap URL.
pub fn robots_txt() -> String {
    let base = std::env::var("SITEMAP_BASE_URL").unwrap_or_else(|_| "/".to_string());
    let base = base.trim_end_matches('/');
    format!("User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n", base)
}

/// Fetch product slugs from gRPC and render as sitemap XML.
pub async fn sitemap_xml() -> Result<String, GqlError> {
    let mut client = connect_grpc_client().await?;
    let req = Request::new(GetSitemapProductUrlsRequest { limit: Some(5000) });
    let res = client
        .get_sitemap_product_urls(req)
        .await
        .map_err(|e: proto::tonic::Status| {
            GqlError::new(e.message(), crate::resolvers::error::Code::Internal)
        })?;
    let inner = res.into_inner();
    let base =
        std::env::var("SITEMAP_BASE_URL").unwrap_or_else(|_| "https://example.com".to_string());
    let base = base.trim_end_matches('/');

    let mut out = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );
    for e in &inner.entries {
        let slug_escaped = e
            .slug
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;");
        let loc = format!("{}/products/{}", base, slug_escaped);
        let lastmod = if e.lastmod.is_empty() {
            String::from("")
        } else {
            format!("  <lastmod>{}</lastmod>\n", e.lastmod)
        };
        out.push_str(&format!(
            "  <url>\n    <loc>{}</loc>\n{}  </url>\n",
            loc, lastmod
        ));
    }
    out.push_str("</urlset>");
    Ok(out)
}
