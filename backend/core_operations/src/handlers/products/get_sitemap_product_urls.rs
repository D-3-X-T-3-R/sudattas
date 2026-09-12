//! P2 SEO: return product slug + lastmod for sitemap generation.

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::products;
use proto::proto::core::{
    GetSitemapProductUrlsRequest, GetSitemapProductUrlsResponse, SitemapEntry,
};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use tonic::{Request, Response, Status};

const DEFAULT_LIMIT: u64 = 5000;

pub async fn get_sitemap_product_urls(
    txn: &DatabaseTransaction,
    request: Request<GetSitemapProductUrlsRequest>,
) -> Result<Response<GetSitemapProductUrlsResponse>, Status> {
    let req = request.into_inner();
    let limit = req
        .limit
        .map(|l| l as u64)
        .unwrap_or(DEFAULT_LIMIT)
        .min(10_000);

    // Resolved against the DB rather than a hardcoded id — see product_state::get_status_id.
    // If "active" doesn't resolve (e.g. a re-seed changed codes), fail closed with an empty
    // sitemap rather than silently submitting every product, drafts included, to Google.
    let Some(active_status_id) = crate::product_state::get_status_id(txn, "active").await? else {
        return Ok(Response::new(GetSitemapProductUrlsResponse {
            entries: vec![],
        }));
    };

    let rows = products::Entity::find()
        .filter(products::Column::Slug.is_not_null())
        .filter(products::Column::ProductStatusId.eq(active_status_id))
        .order_by_desc(products::Column::UpdatedAt)
        .limit(limit)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let entries: Vec<SitemapEntry> =
        rows.into_iter()
            .filter_map(|p| {
                p.slug.as_ref().map(|slug| SitemapEntry {
                    slug: slug.clone(),
                    lastmod: p.updated_at.map(|t| t.to_rfc3339()).unwrap_or_else(|| {
                        p.created_at.map(|t| t.to_rfc3339()).unwrap_or_default()
                    }),
                })
            })
            .collect();

    Ok(Response::new(GetSitemapProductUrlsResponse { entries }))
}
