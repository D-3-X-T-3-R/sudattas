//! P2 Recommendations: return related products for a product.
//! Schema no longer has product_related table; returns empty list. Can be extended with same-category or other logic.

use proto::proto::core::{GetRelatedProductsRequest, ProductsResponse};
use sea_orm::DatabaseTransaction;
use tonic::{Request, Response, Status};

pub async fn get_related_products(
    _txn: &DatabaseTransaction,
    request: Request<GetRelatedProductsRequest>,
) -> Result<Response<ProductsResponse>, Status> {
    let _req = request.into_inner();
    // No product_related table in schema; return empty. Optional: same category or similar.
    Ok(Response::new(ProductsResponse { items: vec![] }))
}
