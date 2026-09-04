use super::Context;
use crate::resolvers::{
    cart::{self, schema::Cart},
    category::{
        self,
        schema::{Category, SearchCategory},
    },
    coupons::{
        self,
        schema::{Coupon, CouponAdmin, PublicCoupon, SearchCouponAdminInput, ValidateCoupon},
    },
    inventory::{
        self,
        schema::{InventoryItem, SearchInventoryItem},
    },
    inventory_logs::{
        self,
        schema::{InventoryLog, SearchInventoryLogInput},
    },
    invoices::{self, schema::InvoiceDownload},
    order_events::{self, schema::OrderEvent},
    orders::{
        self,
        schema::{
            CheckoutShippingEstimate, EstimateCheckoutShippingInput, GetOrderStatsInput, Order,
            OrderStats, OrderStatus, SearchOrder,
        },
    },
    payment_intents::{
        self,
        schema::{GetPaymentIntent, PaymentIntent},
    },
    product::{
        self,
        schema::{GetRelatedProducts, Product, SearchProduct},
    },
    product_images::{
        self,
        schema::{GetPresignedUploadUrl, PresignedUploadUrl, ProductImage, SearchProductImage},
    },
    product_moods::{
        self,
        schema::{ProductMood, SearchProductMoodInput},
    },
    refunds::{
        self,
        schema::{GetRefund, Refund},
    },
    returns::{
        self,
        schema::{ReturnRequest, SearchReturnRequestsInput},
    },
    reviews::{
        self,
        schema::{ProductRatingSummary, Review, SearchReview},
    },
    shipments::{
        self,
        schema::{GetShipment, Shipment},
    },
    shipping_addresses::{self, schema::ShippingAddress},
    shipping_methods::{
        self,
        schema::{SearchShippingMethod, ShippingMethod},
    },
    user_pii::{self, schema::UserPiiExport},
    users::{
        self,
        schema::{SearchUserInput, User},
    },
    wishlist::{
        self,
        schema::{SearchWishlistItem, WishlistItem},
    },
};
use juniper::FieldResult;
use juniper::IntoFieldError;

pub struct QueryRoot;

/// Deactivated/suspended accounts (`setUserStatus`) are rejected here, ahead of the specific
/// `jwt_user_id`/`is_admin` check each caller does — see the identical helper in
/// `mutation_root.rs` for the full rationale.
fn require_not_deactivated(context: &Context) -> Result<(), juniper::FieldError> {
    if context.account_deactivated() {
        return Err(juniper::FieldError::new(
            "This account has been deactivated. Contact support if you believe this is a mistake.",
            juniper::Value::null(),
        ));
    }
    Ok(())
}

fn require_jwt(context: &Context) -> Result<&str, juniper::FieldError> {
    require_not_deactivated(context)?;
    context.jwt_user_id().ok_or_else(|| {
        juniper::FieldError::new("Login required for this operation", juniper::Value::null())
    })
}

fn require_customer_actor(context: &Context) -> Result<&str, juniper::FieldError> {
    require_not_deactivated(context)?;
    context.jwt_user_id().ok_or_else(|| {
        juniper::FieldError::new("Login required for this operation", juniper::Value::null())
    })
}

fn require_admin(context: &Context) -> Result<(), juniper::FieldError> {
    require_not_deactivated(context)?;
    if context.is_admin() {
        Ok(())
    } else {
        crate::metrics::record_admin_authz_denied_total();
        let reason = if context.jwt_user_id().is_none() {
            "not_jwt"
        } else if context.admin_resolution_source() == Some("env_fallback") {
            "fallback_not_admin"
        } else {
            "not_admin"
        };
        crate::metrics::record_admin_authz_denied_reason_total(reason);
        Err(juniper::FieldError::new(
            "Admin authorization required",
            juniper::Value::null(),
        ))
    }
}

pub(crate) async fn ensure_customer_can_access_order(
    context: &Context,
    order_id: &str,
) -> Result<(), juniper::FieldError> {
    if context.is_admin() {
        return Ok(());
    }
    let uid = context.user_id().ok_or_else(|| {
        juniper::FieldError::new(
            "Authentication required for this operation",
            juniper::Value::null(),
        )
    })?;
    let rows = orders::handlers::search_order(SearchOrder {
        user_id: uid.to_string(),
        order_date_start: None,
        order_date_end: None,
        status_id: None,
        order_id: Some(order_id.to_string()),
        limit: Some("1".to_string()),
        offset: Some("0".to_string()),
    })
    .await
    .map_err(|e| e.into_field_error())?;

    if rows.is_empty() {
        return Err(juniper::FieldError::new(
            "Order not found for current user",
            juniper::Value::null(),
        ));
    }
    Ok(())
}

/// Minimal auth capability info; uses Context fields so they are not reported as dead code.
#[derive(juniper::GraphQLObject)]
struct AuthInfo {
    /// Whether session-based (guest) auth is enabled (REDIS_URL configured).
    session_enabled: bool,
    /// Number of JWKS keys loaded for JWT validation.
    jwks_key_count: i32,
    /// Current request’s user ID (JWT or session), if any.
    current_user_id: Option<String>,
    /// True when the current JWT-authenticated identity's account is deactivated/suspended.
    /// Deliberately readable even for a deactivated account — every other JWT-gated query/mutation
    /// rejects them via `require_jwt`/`jwt_user_id()`, so the frontend needs one ungated way to
    /// learn "you're deactivated" in order to show that message instead of a wall of errors.
    account_deactivated: bool,
}

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    /// Returns the current API version string.
    ///
    /// Versioning strategy:
    /// - The GraphQL endpoint is versioned at the URL level (`/v2`).
    /// - Breaking schema changes increment the URL path (→ `/v3`).
    /// - Non-breaking additions (new fields, optional args) are done in-place.
    /// - The gRPC proto package is `proto.core`; breaking proto changes bump the package name.
    /// - Deprecated fields carry `@deprecated` before removal.
    fn api_version() -> &'static str {
        "2.0.0"
    }

    /// Auth capabilities and current identity for this request.
    fn auth_info(context: &Context) -> AuthInfo {
        AuthInfo {
            session_enabled: context.redis_url.is_some(),
            jwks_key_count: context.jwks().keys.len() as i32,
            current_user_id: context.user_id().map(|s| s.to_string()),
            account_deactivated: context.account_deactivated(),
        }
    }

    // Cart
    #[instrument(err, ret)]
    async fn get_cart_items(
        context: &Context,
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> FieldResult<Vec<Cart>> {
        let (resolved_user_id, resolved_session_id) = if context.is_admin() {
            (user_id, session_id)
        } else if let Some(uid) = context.jwt_user_id() {
            (Some(uid.to_string()), None)
        } else if matches!(&context.auth, Some(super::AuthSource::Session(_))) {
            (None, context.guest_session_id().map(|sid| sid.to_string()))
        } else {
            (user_id, session_id)
        };

        cart::handlers::get_cart_items(resolved_user_id, resolved_session_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product
    #[instrument(err, ret)]
    async fn search_product(search: SearchProduct) -> FieldResult<Vec<Product>> {
        product::handlers::search_product(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product moods (admin: list moods for dropdown)
    #[instrument(err, ret)]
    async fn search_product_mood(input: SearchProductMoodInput) -> FieldResult<Vec<ProductMood>> {
        product_moods::handlers::search_product_mood(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Distinct moods from the newest products (storefront Shop by mood).
    #[instrument(err, ret)]
    async fn shop_highlight_moods(
        recent_product_limit: Option<i32>,
        max_moods: Option<i32>,
    ) -> FieldResult<Vec<ProductMood>> {
        product_moods::handlers::shop_highlight_moods(recent_product_limit, max_moods)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// P2 Recommendations: get related products for a given product.
    #[instrument(err, ret)]
    async fn get_related_products(input: GetRelatedProducts) -> FieldResult<Vec<Product>> {
        product::handlers::get_related_products(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // ProductImages
    #[instrument(err, ret)]
    async fn search_product_image(search: SearchProductImage) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::search_product_image(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Category
    #[instrument(err, ret)]
    async fn search_category(search: SearchCategory) -> FieldResult<Vec<Category>> {
        category::handlers::search_category(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order
    #[instrument(err, ret)]
    async fn search_order(context: &Context, mut search: SearchOrder) -> FieldResult<Vec<Order>> {
        if !context.is_admin() {
            let uid = require_jwt(context)?.to_string();
            if !search.user_id.is_empty() && search.user_id != uid {
                return Err(juniper::FieldError::new(
                    "Customers can only query their own orders",
                    juniper::Value::null(),
                ));
            }
            search.user_id = uid;
        }
        orders::handlers::search_order(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Aggregated order/revenue/customer stats for the admin dashboard (admin-only).
    #[instrument(err, ret)]
    async fn order_stats(context: &Context, input: GetOrderStatsInput) -> FieldResult<OrderStats> {
        require_admin(context)?;
        orders::handlers::get_order_stats(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn get_order_invoice_download(
        context: &Context,
        order_id: String,
    ) -> FieldResult<InvoiceDownload> {
        if !context.is_admin() {
            let _ = require_customer_actor(context)?;
            ensure_customer_can_access_order(context, order_id.as_str()).await?;
        }
        invoices::handlers::get_order_invoice_download(order_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    async fn search_order_status(context: &Context) -> FieldResult<Vec<OrderStatus>> {
        let _ = require_jwt(context)?;
        orders::handlers::search_order_status()
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn estimate_checkout_shipping(
        context: &Context,
        input: EstimateCheckoutShippingInput,
    ) -> FieldResult<CheckoutShippingEstimate> {
        let uid = require_jwt(context)?.to_string();
        orders::handlers::estimate_checkout_shipping(input, uid, context.request_id())
            .await
            .map_err(|e| e.into_field_error())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn search_wishlist_item(
        context: &Context,
        mut search: SearchWishlistItem,
    ) -> FieldResult<Vec<WishlistItem>> {
        if !context.is_admin() {
            let uid = require_jwt(context)?.to_string();
            if !search.user_id.is_empty() && search.user_id != uid {
                return Err(juniper::FieldError::new(
                    "Customers can only query their own wishlist",
                    juniper::Value::null(),
                ));
            }
            search.user_id = uid;
        }
        wishlist::handlers::search_wishlist_item(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn get_payment_intent(
        context: &Context,
        input: GetPaymentIntent,
    ) -> FieldResult<Vec<PaymentIntent>> {
        let customer_user_id = if context.is_admin() {
            None
        } else {
            Some(require_customer_actor(context)?.to_string())
        };

        if let (Some(uid), Some(order_id)) = (&customer_user_id, input.order_id.as_deref()) {
            let rows = orders::handlers::search_order(SearchOrder {
                user_id: uid.clone(),
                order_date_start: None,
                order_date_end: None,
                status_id: None,
                order_id: Some(order_id.to_string()),
                limit: Some("1".to_string()),
                offset: Some("0".to_string()),
            })
            .await
            .map_err(|e| e.into_field_error())?;
            if rows.is_empty() {
                return Ok(Vec::new());
            }
        }

        let rows = payment_intents::handlers::get_payment_intent(input)
            .await
            .map_err(|e| e.into_field_error())?;

        if let Some(uid) = customer_user_id {
            let mut filtered = Vec::with_capacity(rows.len());
            for row in rows {
                if row.user_id.as_deref() == Some(uid.as_str()) {
                    filtered.push(row);
                    continue;
                }
                if let Some(oid) = row.order_id.as_deref() {
                    let access = orders::handlers::search_order(SearchOrder {
                        user_id: uid.clone(),
                        order_date_start: None,
                        order_date_end: None,
                        status_id: None,
                        order_id: Some(oid.to_string()),
                        limit: Some("1".to_string()),
                        offset: Some("0".to_string()),
                    })
                    .await
                    .map_err(|e| e.into_field_error())?;
                    if !access.is_empty() {
                        filtered.push(row);
                    }
                }
            }
            Ok(filtered)
        } else {
            Ok(rows)
        }
    }

    // Shipments
    #[instrument(err, ret)]
    async fn get_shipment(context: &Context, input: GetShipment) -> FieldResult<Vec<Shipment>> {
        if let Some(order_id) = input.order_id.as_deref() {
            ensure_customer_can_access_order(context, order_id).await?;
        } else if !context.is_admin() {
            return Err(juniper::FieldError::new(
                "order_id is required for customer shipment lookups",
                juniper::Value::null(),
            ));
        }
        shipments::handlers::get_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Coupons
    #[instrument(err, ret)]
    async fn validate_coupon(context: &Context, input: ValidateCoupon) -> FieldResult<Vec<Coupon>> {
        let _ = require_customer_actor(context)?;
        coupons::handlers::validate_coupon(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Currently-usable coupons (active, in date window, not exhausted) for a "available offers"
    /// list — so customers aren't limited to codes they already know from elsewhere.
    #[instrument(err, ret)]
    async fn list_active_coupons(context: &Context) -> FieldResult<Vec<PublicCoupon>> {
        let _ = require_customer_actor(context)?;
        coupons::handlers::list_active_coupons()
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: list/search coupons.
    #[instrument(err, ret)]
    async fn search_coupon_admin(
        context: &Context,
        input: SearchCouponAdminInput,
    ) -> FieldResult<Vec<CouponAdmin>> {
        require_admin(context)?;
        coupons::handlers::search_coupon_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Reviews
    #[instrument(err, ret)]
    async fn search_review(input: SearchReview) -> FieldResult<Vec<Review>> {
        reviews::handlers::search_review(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Server-computed star rating aggregate for a product (ceil-rounded average + count).
    /// Public, same as search_review — no auth required to view a product's rating.
    #[instrument(err, ret)]
    async fn product_rating_summary(product_id: String) -> FieldResult<ProductRatingSummary> {
        reviews::handlers::get_product_rating_summary(product_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory
    #[instrument(err, ret)]
    async fn search_inventory_item(
        context: &Context,
        input: SearchInventoryItem,
    ) -> FieldResult<Vec<InventoryItem>> {
        require_admin(context)?;
        inventory::handlers::search_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory logs
    #[instrument(err, ret)]
    async fn search_inventory_log(
        context: &Context,
        input: SearchInventoryLogInput,
    ) -> FieldResult<Vec<InventoryLog>> {
        require_admin(context)?;
        inventory_logs::handlers::search_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product Images — R2 presigned upload
    #[instrument(err, ret)]
    async fn get_presigned_upload_url(
        context: &Context,
        input: GetPresignedUploadUrl,
    ) -> FieldResult<Vec<PresignedUploadUrl>> {
        require_admin(context)?;
        product_images::handlers::get_presigned_upload_url(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order Events
    #[instrument(err, ret)]
    async fn get_order_events(context: &Context, order_id: String) -> FieldResult<Vec<OrderEvent>> {
        ensure_customer_can_access_order(context, order_id.as_str()).await?;
        order_events::handlers::get_order_events(order_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Refunds
    #[instrument(err, ret)]
    async fn get_refunds(context: &Context, input: GetRefund) -> FieldResult<Vec<Refund>> {
        if let Some(order_id) = input.order_id.as_deref() {
            ensure_customer_can_access_order(context, order_id).await?;
        } else if !context.is_admin() {
            return Err(juniper::FieldError::new(
                "order_id is required for customer refund lookups",
                juniper::Value::null(),
            ));
        }
        refunds::handlers::get_refunds(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_return_requests(
        context: &Context,
        mut input: SearchReturnRequestsInput,
    ) -> FieldResult<Vec<ReturnRequest>> {
        if !context.is_admin() {
            let uid = require_customer_actor(context)?.to_string();
            if input.user_id.as_deref().is_some_and(|v| v != uid) {
                return Err(juniper::FieldError::new(
                    "Customers can only query their own returns",
                    juniper::Value::null(),
                ));
            }
            input.user_id = Some(uid);
            if let Some(order_id) = input.order_id.as_deref() {
                ensure_customer_can_access_order(context, order_id).await?;
            }
        }

        returns::handlers::search_return_requests(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order Events search (admin audit log)
    #[instrument(err, ret)]
    async fn search_order_events(
        context: &Context,
        order_id: Option<String>,
        limit: Option<String>,
        offset: Option<String>,
    ) -> FieldResult<Vec<OrderEvent>> {
        if !context.is_admin() {
            let oid = order_id.as_deref().ok_or_else(|| {
                juniper::FieldError::new(
                    "order_id is required for customer event queries",
                    juniper::Value::null(),
                )
            })?;
            ensure_customer_can_access_order(context, oid).await?;
        }
        order_events::handlers::search_order_events(
            crate::resolvers::order_events::schema::SearchOrderEvents {
                order_id,
                limit,
                offset,
            },
        )
        .await
        .map_err(|e| e.into_field_error())
    }

    // Shipping methods
    #[instrument(err, ret)]
    async fn search_shipping_method(
        input: SearchShippingMethod,
    ) -> FieldResult<Vec<ShippingMethod>> {
        shipping_methods::handlers::search_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping addresses
    #[instrument(err, ret)]
    async fn get_shipping_addresses(context: &Context) -> FieldResult<Vec<ShippingAddress>> {
        let uid = require_jwt(context)?.to_string();
        let rows = shipping_addresses::handlers::get_shipping_addresses()
            .await
            .map_err(|e| e.into_field_error())?;

        if context.is_admin() {
            return Ok(rows);
        }
        Ok(rows
            .into_iter()
            .filter(|a| a.user_id.as_deref() == Some(uid.as_str()))
            .collect())
    }

    // P2 Data retention: export current user's PII (no password)
    #[instrument(err, ret)]
    async fn export_my_pii(context: &Context) -> FieldResult<UserPiiExport> {
        user_pii::handlers::export_my_pii(context)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Users (admin/user lookup)
    #[instrument(err, ret)]
    async fn search_user(context: &Context, input: SearchUserInput) -> FieldResult<Vec<User>> {
        require_admin(context)?;
        users::handlers::search_user(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin lookup of another customer's PII by id — the admin-facing counterpart to
    /// `exportMyPii`, which is deliberately self-scoped and can't be reused for this.
    #[instrument(err, ret)]
    async fn admin_export_user_pii(
        context: &Context,
        user_id: String,
    ) -> FieldResult<UserPiiExport> {
        require_admin(context)?;
        user_pii::handlers::admin_export_user_pii(context, user_id)
            .await
            .map_err(|e| e.into_field_error())
    }
}
