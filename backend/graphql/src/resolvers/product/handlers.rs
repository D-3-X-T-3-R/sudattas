use proto::proto::core::{
    CreateProductRequest, DeleteProductRequest, GetProductsByIdRequest, GetRelatedProductsRequest,
    SearchInventoryItemRequest, SearchProductRequest, SearchProductVariantRequest,
    SearchSizeRequest, UpdateProductRequest,
};

use tracing::instrument;

use super::schema::{
    GetRelatedProducts, NewProduct, Product, ProductMutation, ProductVariantStock, SearchProduct,
};
use crate::resolvers::{
    convert,
    error::GqlError,
    utils::{connect_grpc_client, parse_i64, to_i64, to_option_i64},
};
use crate::validation::{
    validate_amount_paise, validate_non_negative_amount_paise, validate_sku_slug,
};

#[instrument]
pub(crate) async fn create_product(product: NewProduct) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let name = product.name;
    let price_paise = parse_i64(&product.price_paise, "price_paise")?;
    validate_amount_paise(price_paise, "price_paise")?;
    let description = product.description;
    let category_id = parse_i64(&product.category_id, "category id")?;
    if let Some(ref sku) = product.sku {
        validate_sku_slug(sku, "sku")?;
    }
    if let Some(ref slug) = product.slug {
        validate_sku_slug(slug, "slug")?;
    }
    let product_status_id = product
        .product_status_id
        .as_ref()
        .and_then(|s| s.parse().ok());

    let response = client
        .create_product(CreateProductRequest {
            name,
            description: Some(description),
            price_paise,
            category_id,
            sku: product.sku,
            slug: product.slug,
            fabric: product.fabric,
            weave: product.weave,
            occasion: product.occasion,
            has_blouse_piece: product.has_blouse_piece,
            care_instructions: product.care_instructions.clone(),
            product_status_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn search_product(search: SearchProduct) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let starting_price_paise = match search.starting_price_paise.as_ref() {
        Some(v) => {
            let parsed = parse_i64(v, "starting_price_paise")?;
            validate_non_negative_amount_paise(parsed, "starting_price_paise")?;
            Some(parsed)
        }
        None => None,
    };
    let ending_price_paise = match search.ending_price_paise.as_ref() {
        Some(v) => {
            let parsed = parse_i64(v, "ending_price_paise")?;
            validate_non_negative_amount_paise(parsed, "ending_price_paise")?;
            Some(parsed)
        }
        None => None,
    };
    if let (Some(min), Some(max)) = (starting_price_paise, ending_price_paise) {
        if min > max {
            return Err(GqlError::new(
                "starting_price_paise must be less than or equal to ending_price_paise",
                crate::resolvers::error::Code::InvalidArgument,
            ));
        }
    }

    let limit = crate::graphql_limits::cap_page_size(to_option_i64(search.limit));
    let response = client
        .search_product(SearchProductRequest {
            name: search.name,
            description: search.description,
            starting_price_paise,
            ending_price_paise,
            category_id: to_option_i64(search.category_id),
            product_id: to_option_i64(search.product_id),
            limit,
            offset: to_option_i64(search.offset),
            fabric: search.fabric,
            weave: search.weave,
            occasion: search.occasion,
            product_status_id: to_option_i64(search.product_status_id),
            mood_id: to_option_i64(search.mood_id),
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn delete_product(product_id: String) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;

    let product_id = parse_i64(&product_id, "product id")?;

    let response = client
        .delete_product(DeleteProductRequest { product_id })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

#[instrument]
pub(crate) async fn update_product(product: ProductMutation) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let price_paise = parse_i64(&product.price_paise, "price_paise")?;
    validate_amount_paise(price_paise, "price_paise")?;
    if let Some(ref sku) = product.sku {
        validate_sku_slug(sku, "sku")?;
    }
    if let Some(ref slug) = product.slug {
        validate_sku_slug(slug, "slug")?;
    }

    let product_status_id = product
        .product_status_id
        .as_ref()
        .and_then(|s| s.parse().ok());

    let response = client
        .update_product(UpdateProductRequest {
            name: product.name,
            description: Some(product.description),
            price_paise,
            category_id: parse_i64(&product.category_id, "category id")?,
            product_id: to_i64(product.product_id),
            sku: product.sku,
            slug: product.slug,
            fabric: product.fabric,
            weave: product.weave,
            occasion: product.occasion,
            has_blouse_piece: product.has_blouse_piece,
            care_instructions: product.care_instructions.clone(),
            product_status_id,
        })
        .await?;

    Ok(response
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

/// Resolve product(s) for a variant (e.g. for cart/order detail line). Uses variant_id -> product_id.
#[instrument]
pub(crate) async fn get_products_for_variant(variant_id: &str) -> Result<Vec<Product>, GqlError> {
    let variant_id = parse_i64(variant_id, "variant id")?;
    let mut client = connect_grpc_client().await?;
    let variant_resp = client
        .search_product_variant(SearchProductVariantRequest {
            variant_id,
            product_id: None,
        })
        .await?;
    let items = variant_resp.into_inner().items;
    let product_ids: Vec<i64> = items.into_iter().map(|v| v.product_id).collect();
    if product_ids.is_empty() {
        return Ok(Vec::new());
    }
    let resp = client
        .get_products_by_id(GetProductsByIdRequest { product_ids })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

/// P2 Recommendations: fetch related products for a given product.
#[instrument]
pub(crate) async fn get_related_products(
    input: GetRelatedProducts,
) -> Result<Vec<Product>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let product_id = parse_i64(&input.product_id, "product_id")?;
    // Keep related-products pagination bounded like other list resolvers.
    let limit = crate::graphql_limits::cap_page_size(to_option_i64(input.limit));
    let resp = client
        .get_related_products(GetRelatedProductsRequest { product_id, limit })
        .await?;
    Ok(resp
        .into_inner()
        .items
        .into_iter()
        .map(convert::product_response_to_gql)
        .collect())
}

/// Compute total stock for a product as the sum of inventory quantities across all its variants.
#[instrument]
pub(crate) async fn get_stock_for_product(product_id: &str) -> Result<Option<String>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let product_id_i64 = parse_i64(product_id, "product id")?;

    let variants_resp = client
        .search_product_variant(SearchProductVariantRequest {
            variant_id: 0,
            product_id: Some(product_id_i64),
        })
        .await?;
    let variants = variants_resp.into_inner().items;
    let variant_ids: Vec<i64> = variants.into_iter().map(|v| v.variant_id).collect();

    if variant_ids.is_empty() {
        return Ok(None);
    }

    let mut total: i64 = 0;
    for variant_id in variant_ids {
        let inventory_resp = client
            .search_inventory_item(SearchInventoryItemRequest {
                inventory_id: None,
                variant_id: Some(variant_id),
            })
            .await?;
        for item in inventory_resp.into_inner().items {
            total += item.quantity_available;
        }
    }

    if total == 0 {
        Ok(None)
    } else {
        Ok(Some(total.to_string()))
    }
}

/// Per-size stock for storefront. Variants with size_id appear per size; variants with no size_id become one "Free Size" entry.
#[instrument]
pub(crate) async fn get_variant_stock_for_product(
    product_id: &str,
) -> Result<Vec<ProductVariantStock>, GqlError> {
    let mut client = connect_grpc_client().await?;
    let product_id_i64 = parse_i64(product_id, "product id")?;

    let variants_resp = client
        .search_product_variant(SearchProductVariantRequest {
            variant_id: 0,
            product_id: Some(product_id_i64),
        })
        .await?;
    let all_variants: Vec<_> = variants_resp.into_inner().items;
    let variants_with_size: Vec<_> = all_variants
        .iter()
        .filter(|v| v.size_id.is_some())
        .cloned()
        .collect();
    let free_size_variants: Vec<_> = all_variants
        .into_iter()
        .filter(|v| v.size_id.is_none())
        .collect();

    // Build size_id -> size_name map (fetch all sizes once)
    let sizes_resp = client.search_size(SearchSizeRequest { size_id: 0 }).await?;
    let size_names: std::collections::HashMap<i64, String> = sizes_resp
        .into_inner()
        .items
        .into_iter()
        .map(|s| (s.size_id, s.size_name))
        .collect();

    let mut out = Vec::new();

    for v in variants_with_size {
        let size_id = v.size_id.expect("filtered to Some");
        let size_name = size_names
            .get(&size_id)
            .cloned()
            .unwrap_or_else(|| size_id.to_string());

        let inv_resp = client
            .search_inventory_item(SearchInventoryItemRequest {
                inventory_id: None,
                variant_id: Some(v.variant_id),
            })
            .await?;
        let quantity: i32 = inv_resp
            .into_inner()
            .items
            .into_iter()
            .map(|i| i.quantity_available)
            .sum::<i64>()
            .clamp(0, i64::from(i32::MAX)) as i32;

        out.push(ProductVariantStock {
            variant_id: v.variant_id.to_string(),
            size_id: size_id.to_string(),
            size_name,
            quantity,
        });
    }

    // Free size: single entry using first variant with no size_id
    if let Some(first) = free_size_variants.first() {
        let inv_resp = client
            .search_inventory_item(SearchInventoryItemRequest {
                inventory_id: None,
                variant_id: Some(first.variant_id),
            })
            .await?;
        let quantity: i32 = inv_resp
            .into_inner()
            .items
            .into_iter()
            .map(|i| i.quantity_available)
            .sum::<i64>()
            .clamp(0, i64::from(i32::MAX)) as i32;
        out.push(ProductVariantStock {
            variant_id: first.variant_id.to_string(),
            size_id: "0".to_string(),
            size_name: "Free Size".to_string(),
            quantity,
        });
    }

    Ok(out)
}
