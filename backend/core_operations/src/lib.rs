use core_db_entities::CoreDatabaseConnection;
use handlers::db_errors::{is_deadlock_status, map_db_error_to_status};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

// Phase 1 additions
pub mod auth;
pub mod cancellation_saga;
pub mod money;
pub mod observability;
pub mod order_policy;
pub mod product_state;
pub mod razorpay;
pub mod services;

static DOTENV_LOADED: OnceLock<()> = OnceLock::new();
static GRPC_AUTH_RELAXED_WARNED: OnceLock<()> = OnceLock::new();

/// How many times to retry a transaction that failed with an InnoDB deadlock before giving up and
/// returning the error to the caller. Applied to the gRPC methods identified as taking part in a
/// known conflicting lock-order pair (see db_errors::is_deadlock_status doc comment) — not applied
/// codebase-wide, since retrofitting retry onto every transition_order_status call site would be a
/// much larger, unaudited change (tracked separately as ISSUE-007 option A). `pub(crate)` so
/// `procedures::orders::place_order` (which owns its own deadlock-retry loop around its write
/// phase) can reuse the same policy value instead of duplicating it.
pub(crate) const DEADLOCK_MAX_RETRIES: u32 = 3;

pub fn load_env_once() {
    DOTENV_LOADED.get_or_init(|| {
        let candidates = [
            PathBuf::from("..").join(".env"),
            PathBuf::from(".env"),
            PathBuf::from("backend").join(".env"),
        ];
        for candidate in candidates {
            if candidate.exists() {
                let _ = dotenvy::from_path(candidate);
                break;
            }
        }
    });
}

use proto::proto::core::{
    grpc_services_server::GrpcServices, AddWishlistItemRequest, AdminMarkExchangeReceivedRequest,
    AdminMarkOrderDeliveredRequest, AdminMarkOrderDeliveredResponse, AdminMarkOrderShippedRequest,
    AdminMarkOrderShippedResponse, AdminMarkReturnReceivedRequest,
    AdminUpdateExchangeStatusRequest, AdminUpdateReturnStatusRequest,
    AdminUpdateReviewStatusRequest, AdminUpdateReviewStatusResponse, ApplyCouponRequest,
    ArchiveProductRequest, CancelOrderItemsRequest, CapturePaymentRequest, CartItemsResponse,
    CategoriesResponse, ColorsResponse, ConfirmImageUploadRequest, CouponsAdminResponse,
    CouponsResponse, CreateCartItemRequest, CreateCategoryRequest, CreateColorRequest,
    CreateCouponRequest, CreateEventLogRequest, CreateFabricRequest, CreateInventoryItemRequest,
    CreateInventoryLogRequest, CreateNewsletterSubscriberRequest, CreateOccasionRequest,
    CreateOrderDetailsRequest, CreateOrderEventRequest, CreateOrderRequest,
    CreatePaymentIntentRequest, CreateProductMoodMappingRequest, CreateProductMoodRequest,
    CreateProductRequest, CreateProductVariantRequest, CreateRefundRequest, CreateReviewRequest,
    CreateShipmentRequest, CreateShippingAddressRequest, CreateShippingMethodRequest,
    CreateSizeRequest, CreateTransactionRequest, CreateUserActivityRequest, CreateUserRequest,
    CreateUserRoleRequest, CreateWeaveRequest, DeleteCartItemRequest, DeleteCategoryRequest,
    DeleteColorRequest, DeleteCouponAdminRequest, DeleteEventLogRequest, DeleteFabricRequest,
    DeleteInventoryItemRequest, DeleteInventoryLogRequest, DeleteNewsletterSubscriberRequest,
    DeleteOccasionRequest, DeleteOrderRequest, DeleteProductImageRequest,
    DeleteProductMoodMappingRequest, DeleteProductMoodRequest, DeleteProductRequest,
    DeleteProductVariantRequest, DeleteReviewRequest, DeleteShippingAddressRequest,
    DeleteShippingMethodRequest, DeleteSizeRequest, DeleteTransactionRequest,
    DeleteUserActivityRequest, DeleteUserRequest, DeleteUserRoleRequest, DeleteWeaveRequest,
    DeleteWishlistItemRequest, EnqueueAbandonedCartRequest, EnqueueAbandonedCartResponse,
    EstimateCheckoutShippingRequest, EstimateCheckoutShippingResponse, EventLogsResponse,
    ExchangeRequestsResponse, FabricsResponse, GetCartItemsRequest, GetOrderEventsRequest,
    GetOrderInvoiceDownloadRequest, GetOrderInvoiceDownloadResponse, GetOrderInvoiceRequest,
    GetOrderStatsRequest, GetOrderStatsResponse, GetPaymentIntentRequest,
    GetPresignedUploadUrlRequest, GetProductsByIdRequest, GetRefundsRequest,
    GetRelatedProductsRequest, GetShipmentRequest, GetShippingAddressRequest,
    GetSitemapProductUrlsRequest, GetSitemapProductUrlsResponse, GetUserPiiExportRequest,
    GetUserPiiExportResponse, IngestWebhookRequest, InventoryItemsResponse, InventoryLogsResponse,
    InvoiceResponse, ListActiveCouponsRequest, MergeCartRequest, NewsletterCampaignsResponse,
    NewsletterSubscribersResponse, OccasionsResponse, OrderDetailsResponse, OrderEventsResponse,
    OrderStatusesResponse, OrdersResponse, PaymentIntentsResponse, PermanentlyDeleteProductRequest,
    PermanentlyDeleteProductResponse, PlaceOrderAdminRequest, PlaceOrderRequest,
    PresignedUploadUrlResponse, ProductImagesResponse, ProductMoodMappingsResponse,
    ProductMoodsResponse, ProductRatingSummaryRequest, ProductRatingSummaryResponse,
    ProductVariantsResponse, ProductsResponse, PublicCouponsResponse, ReadinessRequest,
    ReadinessResponse, RecordSecurityAuditRequest, RecordSecurityAuditResponse,
    RefundAttemptsResponse, RefundsResponse, RequestExchangeRequest, RequestReturnRequest,
    ResolveNeedsReviewRequest, ResolveNeedsReviewResponse, ResolveRefundAttemptNeedsReviewRequest,
    ResolveRefundAttemptNeedsReviewResponse, ReturnRequestsResponse, ReviewsResponse,
    SearchCategoryRequest, SearchColorRequest, SearchCouponAdminRequest, SearchEventLogRequest,
    SearchExchangeRequestsRequest, SearchFabricRequest, SearchInventoryItemRequest,
    SearchInventoryLogRequest, SearchNewsletterCampaignRequest, SearchNewsletterSubscriberRequest,
    SearchOccasionRequest, SearchOrderDetailRequest, SearchOrderEventsRequest, SearchOrderRequest,
    SearchOrderStatusRequest, SearchProductImageRequest, SearchProductMoodMappingRequest,
    SearchProductMoodRequest, SearchProductRequest, SearchProductVariantRequest,
    SearchRefundAttemptsRequest, SearchReturnRequestsRequest, SearchReviewRequest,
    SearchShippingMethodRequest, SearchSizeRequest, SearchTransactionRequest,
    SearchUserActivityRequest, SearchUserRequest, SearchUserRoleRequest, SearchWeaveRequest,
    SearchWishlistItemRequest, SendNewsletterCampaignRequest, SetUserStatusRequest,
    ShipmentsResponse, ShippingAddressesResponse, ShippingMethodsResponse,
    ShopHighlightMoodsRequest, ShopHighlightMoodsResponse, SizesResponse,
    SyncOrderShipmentsFromShiprocketRequest, SyncOrderShipmentsFromShiprocketResponse,
    SyncProductImagesRequest, TransactionsResponse, UnsubscribeNewsletterByTokenRequest,
    UpdateCartItemRequest, UpdateCategoryRequest, UpdateColorRequest, UpdateCouponRequest,
    UpdateEventLogRequest, UpdateFabricRequest, UpdateInventoryItemRequest,
    UpdateInventoryLogRequest, UpdateNewsletterSubscriberRequest, UpdateOccasionRequest,
    UpdateOrderDetailRequest, UpdateOrderRequest, UpdatePickupTargetRequest,
    UpdatePickupTargetResponse, UpdateProductImageRequest, UpdateProductMoodRequest,
    UpdateProductRequest, UpdateProductVariantRequest, UpdateReviewRequest, UpdateShipmentRequest,
    UpdateShippingAddressRequest, UpdateShippingMethodRequest, UpdateSizeRequest,
    UpdateTransactionRequest, UpdateUserActivityRequest, UpdateUserRequest, UpdateUserRoleRequest,
    UpdateWeaveRequest, UserActivitiesResponse, UserRolesResponse, UsersResponse,
    ValidateCouponRequest, VerifyRazorpayPaymentRequest, VerifyRazorpayPaymentResponse,
    WeavesResponse, WebhookEventsResponse, WishlistItemsResponse,
};

use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

pub mod handlers;
pub mod integrations;
pub mod notifications;
pub mod order_state_machine;
pub mod procedures;
pub mod schema_guard;

#[derive(Default, Debug)]
pub struct MyGRPCServices {
    db: Option<Arc<CoreDatabaseConnection>>,
    session_manager: Option<auth::session::SessionManager>,
}

/// Names treated as safe-to-relax (permissive CORS/webhook/auth defaults). Anything
/// else that APP_ENV/RUST_ENV/NODE_ENV is explicitly set to — including a typo like
/// "prod" or "stage", or an unrecognized value — is treated as production so a
/// misconfigured env var fails closed instead of silently disabling strict startup
/// validation and gRPC auth. Only leaving these vars entirely unset (the local-dev
/// convention in this repo's .env.example) is treated as safe-to-relax.
const KNOWN_NON_PRODUCTION_ENV_VALUES: &[&str] = &["development", "dev", "local", "test"];

fn is_production_env() -> bool {
    let values: Vec<String> = ["APP_ENV", "RUST_ENV", "NODE_ENV"]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect();

    if values.is_empty() {
        return false;
    }

    values
        .iter()
        .any(|value| !KNOWN_NON_PRODUCTION_ENV_VALUES.contains(&value.as_str()))
}

fn parse_bool_env_or_default(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => {
                tracing::warn!("{key} has invalid value; using default for gRPC auth strictness");
                default
            }
        },
        Err(_) => default,
    }
}

fn is_grpc_auth_strict_mode() -> bool {
    is_production_env() || parse_bool_env_or_default("STRICT_STARTUP_VALIDATION", false)
}

fn configured_grpc_auth_token() -> Option<String> {
    std::env::var("GRPC_AUTH_TOKEN")
        .ok()
        .map(|raw| raw.trim().to_owned())
        .filter(|token| !token.is_empty())
}

fn warn_grpc_auth_relaxed_mode() {
    if GRPC_AUTH_RELAXED_WARNED.set(()).is_ok() {
        tracing::warn!(
            "GRPC_AUTH_TOKEN is not configured; allowing unauthenticated gRPC requests because strict startup validation is disabled"
        );
    }
}

/// gRPC interceptor that enforces bearer-token auth.
///
/// - Strict/prod mode (`STRICT_STARTUP_VALIDATION=true` or production env) is fail-closed.
/// - Relaxed non-production mode allows pass-through only when `GRPC_AUTH_TOKEN` is not configured.
/// - All authenticated calls must supply `authorization: Bearer <GRPC_AUTH_TOKEN>`.
#[allow(clippy::result_large_err)]
pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let strict_mode = is_grpc_auth_strict_mode();
    let expected_token = match configured_grpc_auth_token() {
        Some(token) => token,
        None if strict_mode => {
            tracing::warn!(
                "gRPC auth rejected: GRPC_AUTH_TOKEN is missing while strict validation is enabled"
            );
            return Err(Status::unauthenticated(
                "gRPC authentication is not configured",
            ));
        }
        None => {
            warn_grpc_auth_relaxed_mode();
            return Ok(req);
        }
    };

    let provided = req
        .metadata()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(tok) if tok == expected_token => Ok(req),
        Some(_) => {
            tracing::warn!("gRPC auth rejected: invalid token");
            Err(Status::unauthenticated("Invalid authorization token"))
        }
        None => {
            tracing::warn!("gRPC auth rejected: missing authorization header");
            Err(Status::unauthenticated("Missing authorization header"))
        }
    }
}

impl MyGRPCServices {
    /// Initializes the service with a shared DB connection pool. The pool is
    /// constructed once by the caller (see `main.rs`) and passed in — rather than
    /// each of `init()` and every background worker independently calling
    /// `get_db()` and opening its own pool — so the process opens one bounded
    /// connection pool instead of one per component (gRPC service + N workers).
    pub async fn init(&mut self, db: Arc<CoreDatabaseConnection>) -> Result<(), Status> {
        crate::schema_guard::validate_required_schema(&db)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        self.db = Some(db);

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            match auth::session::SessionManager::new(&redis_url, Duration::from_secs(86400)) {
                Ok(sm) => self.session_manager = Some(sm),
                Err(e) => log::warn!("Redis session manager not available: {}", e),
            }
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl GrpcServices for MyGRPCServices {
    // Shipping Address Service
    async fn create_shipping_address(
        &self,
        request: Request<CreateShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_address::create_shipping_address(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_shipping_address(
        &self,
        request: Request<UpdateShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_address::update_shipping_address(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_shipping_address(
        &self,
        request: Request<DeleteShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_address::delete_shipping_address(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_shipping_address(
        &self,
        request: Request<GetShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_address::get_shipping_address(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Cart Services
    async fn create_cart_item(
        &self,
        request: Request<CreateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::create_cart_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_cart_items(
        &self,
        request: Request<GetCartItemsRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::get_cart_items(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_cart_item(
        &self,
        request: Request<UpdateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::update_cart_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_cart_item(
        &self,
        request: Request<DeleteCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::delete_cart_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn merge_cart(
        &self,
        request: Request<MergeCartRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::merge_cart(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn enqueue_abandoned_cart(
        &self,
        request: Request<EnqueueAbandonedCartRequest>,
    ) -> Result<Response<EnqueueAbandonedCartResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;
        handlers::cart::enqueue_abandoned_cart(db, request).await
    }

    // Product Services
    async fn create_product(
        &self,
        request: Request<CreateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::create_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product(
        &self,
        request: Request<SearchProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::search_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_products_by_id(
        &self,
        request: Request<GetProductsByIdRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::get_products_by_id(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_related_products(
        &self,
        request: Request<GetRelatedProductsRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::get_related_products(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_sitemap_product_urls(
        &self,
        request: Request<GetSitemapProductUrlsRequest>,
    ) -> Result<Response<GetSitemapProductUrlsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::get_sitemap_product_urls(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product(
        &self,
        request: Request<DeleteProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::delete_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn archive_product(
        &self,
        request: Request<ArchiveProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::archive_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn permanently_delete_product(
        &self,
        request: Request<PermanentlyDeleteProductRequest>,
    ) -> Result<Response<PermanentlyDeleteProductResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;
        // Manages its own transaction (commits the DB cascade first, then does a best-effort
        // R2 purge afterward) — see procedures::products::permanently_delete_product's doc
        // comment for why this isn't wrapped in the usual single begin/commit here.
        procedures::products::permanently_delete_product(db, request).await
    }

    async fn update_product(
        &self,
        request: Request<UpdateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::update_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // User Services
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let mut res = handlers::users::create_user(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;

        if let Some(sm) = &self.session_manager {
            if let Some(u) = res.get_ref().items.first() {
                let data = auth::session::SessionData {
                    user_id: Some(u.user_id),
                    email: Some(u.email.clone()),
                    ..Default::default()
                };
                match sm.create_session(data).await {
                    Ok(session_id) => {
                        if let Some(first) = res.get_mut().items.first_mut() {
                            first.session_id = Some(session_id);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to create session for new user {}: {}", u.user_id, e);
                    }
                }
            }
        }

        Ok(res)
    }

    async fn search_user(
        &self,
        request: Request<SearchUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::search_user(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::update_user(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::delete_user(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn set_user_status(
        &self,
        request: Request<SetUserStatusRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::set_user_status(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_user_pii_export(
        &self,
        request: Request<GetUserPiiExportRequest>,
    ) -> Result<Response<GetUserPiiExportResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::get_user_pii_export(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn record_security_audit_event(
        &self,
        request: Request<RecordSecurityAuditRequest>,
    ) -> Result<Response<RecordSecurityAuditResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::security::record_security_audit_event(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Category Services
    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::categories::create_category(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_category(
        &self,
        request: Request<SearchCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::categories::search_category(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::categories::update_category(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::categories::delete_category(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Order Services
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::create_order(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("Database not initialized"))?;

        // Durable idempotency for place_order is implemented at the database layer via the
        // idempotency_keys table. procedures::orders::place_order manages its own
        // transactions internally (a short claim/prep transaction, then the Shiprocket and
        // Razorpay calls with no DB connection held, then a write transaction with its own
        // deadlock-retry loop around the inventory-locking step) so this checkout flow never
        // holds a pooled connection open across either external round-trip.
        procedures::orders::place_order(db, request).await
    }

    async fn place_order_admin(
        &self,
        request: Request<PlaceOrderAdminRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("Database not initialized"))?;
        procedures::orders::place_order_admin(db, request).await
    }

    async fn estimate_checkout_shipping(
        &self,
        request: Request<EstimateCheckoutShippingRequest>,
    ) -> Result<Response<EstimateCheckoutShippingResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("Database not initialized"))?;
        // procedures::orders::estimate_checkout_shipping manages its own (read-only) transaction
        // internally so it can commit before its Shiprocket call rather than holding a pooled
        // connection open across that external round-trip.
        procedures::orders::estimate_checkout_shipping(db, request).await
    }

    async fn search_order(
        &self,
        request: Request<SearchOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::search_order(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_order(
        &self,
        request: Request<UpdateOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::update_order(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn admin_mark_order_shipped(
        &self,
        request: Request<AdminMarkOrderShippedRequest>,
    ) -> Result<Response<AdminMarkOrderShippedResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;
        let txn = db.begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::orders::admin_mark_order_shipped(&txn, db, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn admin_mark_order_delivered(
        &self,
        request: Request<AdminMarkOrderDeliveredRequest>,
    ) -> Result<Response<AdminMarkOrderDeliveredResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::admin_mark_order_delivered(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_pickup_target(
        &self,
        request: Request<UpdatePickupTargetRequest>,
    ) -> Result<Response<UpdatePickupTargetResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::update_pickup_target(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_order_status(
        &self,
        request: Request<SearchOrderStatusRequest>,
    ) -> Result<Response<OrderStatusesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::search_order_status(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_order_invoice(
        &self,
        request: Request<GetOrderInvoiceRequest>,
    ) -> Result<Response<InvoiceResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::invoices::get_order_invoice(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_order_invoice_download(
        &self,
        request: Request<GetOrderInvoiceDownloadRequest>,
    ) -> Result<Response<GetOrderInvoiceDownloadResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::invoices::get_order_invoice_download(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_order(
        &self,
        request: Request<DeleteOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::delete_order(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn cancel_order_items(
        &self,
        request: Request<CancelOrderItemsRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let req = request.into_inner();
        let mut attempt: u32 = 0;
        loop {
            let txn = self
                .db
                .as_ref()
                .ok_or_else(|| Status::unavailable("database not initialized"))?
                .begin()
                .await
                .map_err(map_db_error_to_status)?;
            let result =
                match handlers::orders::cancel_order_items(&txn, Request::new(req.clone())).await {
                    Ok(res) => txn
                        .commit()
                        .await
                        .map_err(map_db_error_to_status)
                        .map(|_| res),
                    Err(status) => Err(status),
                };
            match result {
                Err(status) if attempt < DEADLOCK_MAX_RETRIES && is_deadlock_status(&status) => {
                    attempt += 1;
                    tracing::warn!(
                        attempt,
                        "cancel_order_items: retrying after InnoDB deadlock"
                    );
                    continue;
                }
                other => return other,
            }
        }
    }

    async fn request_return(
        &self,
        request: Request<RequestReturnRequest>,
    ) -> Result<Response<ReturnRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::returns::request_return(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_return_requests(
        &self,
        request: Request<SearchReturnRequestsRequest>,
    ) -> Result<Response<ReturnRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::returns::search_return_requests(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn admin_mark_return_received(
        &self,
        request: Request<AdminMarkReturnReceivedRequest>,
    ) -> Result<Response<ReturnRequestsResponse>, Status> {
        let req = request.into_inner();
        let mut attempt: u32 = 0;
        loop {
            let txn = self
                .db
                .as_ref()
                .ok_or_else(|| Status::unavailable("database not initialized"))?
                .begin()
                .await
                .map_err(map_db_error_to_status)?;
            let result = match handlers::returns::admin_mark_return_received(
                &txn,
                Request::new(req.clone()),
            )
            .await
            {
                Ok(res) => txn
                    .commit()
                    .await
                    .map_err(map_db_error_to_status)
                    .map(|_| res),
                Err(status) => Err(status),
            };
            match result {
                Err(status) if attempt < DEADLOCK_MAX_RETRIES && is_deadlock_status(&status) => {
                    attempt += 1;
                    tracing::warn!(
                        attempt,
                        "admin_mark_return_received: retrying after InnoDB deadlock"
                    );
                    continue;
                }
                other => return other,
            }
        }
    }

    async fn admin_update_return_status(
        &self,
        request: Request<AdminUpdateReturnStatusRequest>,
    ) -> Result<Response<ReturnRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::returns::admin_update_return_status(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn request_exchange(
        &self,
        request: Request<RequestExchangeRequest>,
    ) -> Result<Response<ExchangeRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::exchanges::request_exchange(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_exchange_requests(
        &self,
        request: Request<SearchExchangeRequestsRequest>,
    ) -> Result<Response<ExchangeRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::exchanges::search_exchange_requests(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Unlike the other exchange endpoints, this one manages multiple transactions itself
    // (restore stock, then place_order_admin's own transaction, then mark completed) — see the
    // doc comment on handlers::exchanges::admin_mark_exchange_received.
    async fn admin_mark_exchange_received(
        &self,
        request: Request<AdminMarkExchangeReceivedRequest>,
    ) -> Result<Response<ExchangeRequestsResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;
        handlers::exchanges::admin_mark_exchange_received(db, request).await
    }

    async fn admin_update_exchange_status(
        &self,
        request: Request<AdminUpdateExchangeStatusRequest>,
    ) -> Result<Response<ExchangeRequestsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::exchanges::admin_update_exchange_status(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // OrderDetails Services
    async fn create_order_details(
        &self,
        request: Request<CreateOrderDetailsRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_details::create_order_details(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_order_detail(
        &self,
        request: Request<SearchOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_details::search_order_detail(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_order_detail(
        &self,
        request: Request<UpdateOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_details::update_order_detail(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Reviews Services
    async fn create_review(
        &self,
        request: Request<CreateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::create_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_review(
        &self,
        request: Request<SearchReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::search_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_review(
        &self,
        request: Request<UpdateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::update_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_review(
        &self,
        request: Request<DeleteReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::delete_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn admin_update_review_status(
        &self,
        request: Request<AdminUpdateReviewStatusRequest>,
    ) -> Result<Response<AdminUpdateReviewStatusResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::admin_update_review_status(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_product_rating_summary(
        &self,
        request: Request<ProductRatingSummaryRequest>,
    ) -> Result<Response<ProductRatingSummaryResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::get_product_rating_summary(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductImages Services
    async fn search_product_image(
        &self,
        request: Request<SearchProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::search_product_image(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product_image(
        &self,
        request: Request<UpdateProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::update_product_image(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_image(
        &self,
        request: Request<DeleteProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::delete_product_image(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Inventory Services
    async fn create_inventory_item(
        &self,
        request: Request<CreateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory::create_inventory_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_inventory_item(
        &self,
        request: Request<SearchInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory::search_inventory_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_inventory_item(
        &self,
        request: Request<UpdateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory::update_inventory_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_inventory_item(
        &self,
        request: Request<DeleteInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory::delete_inventory_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Wishlist Services
    async fn add_wishlist_item(
        &self,
        request: Request<AddWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::wishlist::add_wishlist_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_wishlist_item(
        &self,
        request: Request<SearchWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::wishlist::search_wishlist_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_wishlist_item(
        &self,
        request: Request<DeleteWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::wishlist::delete_wishlist_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductMoods Services
    async fn create_product_mood(
        &self,
        request: Request<CreateProductMoodRequest>,
    ) -> Result<Response<ProductMoodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_moods::create_product_mood(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_mood(
        &self,
        request: Request<SearchProductMoodRequest>,
    ) -> Result<Response<ProductMoodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_moods::search_product_mood(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn shop_highlight_moods(
        &self,
        request: Request<ShopHighlightMoodsRequest>,
    ) -> Result<Response<ShopHighlightMoodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_moods::shop_highlight_moods(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product_mood(
        &self,
        request: Request<UpdateProductMoodRequest>,
    ) -> Result<Response<ProductMoodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_moods::update_product_mood(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_mood(
        &self,
        request: Request<DeleteProductMoodRequest>,
    ) -> Result<Response<ProductMoodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_moods::delete_product_mood(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ShippingMethods Services
    async fn create_shipping_method(
        &self,
        request: Request<CreateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_methods::create_shipping_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_shipping_method(
        &self,
        request: Request<SearchShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_methods::search_shipping_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_shipping_method(
        &self,
        request: Request<UpdateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_methods::update_shipping_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_shipping_method(
        &self,
        request: Request<DeleteShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_methods::delete_shipping_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // UserRole Services
    async fn create_user_role(
        &self,
        request: Request<CreateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_roles::create_user_role(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_user_role(
        &self,
        request: Request<SearchUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_roles::search_user_role(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_user_role(
        &self,
        request: Request<UpdateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_roles::update_user_role(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_user_role(
        &self,
        request: Request<DeleteUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_roles::delete_user_role(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Transactions Services
    async fn create_transaction(
        &self,
        request: Request<CreateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::transactions::create_transaction(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_transaction(
        &self,
        request: Request<SearchTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::transactions::search_transaction(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_transaction(
        &self,
        request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::transactions::update_transaction(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_transaction(
        &self,
        request: Request<DeleteTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::transactions::delete_transaction(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // NewsletterSubscribers Services
    async fn create_newsletter_subscriber(
        &self,
        request: Request<CreateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::newsletter_subscribers::create_newsletter_subscriber(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_newsletter_subscriber(
        &self,
        request: Request<SearchNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::newsletter_subscribers::search_newsletter_subscriber(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_newsletter_subscriber(
        &self,
        request: Request<UpdateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::newsletter_subscribers::update_newsletter_subscriber(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_newsletter_subscriber(
        &self,
        request: Request<DeleteNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::newsletter_subscribers::delete_newsletter_subscriber(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn unsubscribe_newsletter_by_token(
        &self,
        request: Request<UnsubscribeNewsletterByTokenRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::newsletter_subscribers::unsubscribe_newsletter_by_token(&txn, request)
            .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_newsletter_campaign(
        &self,
        request: Request<SearchNewsletterCampaignRequest>,
    ) -> Result<Response<NewsletterCampaignsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::newsletter_campaigns::search_newsletter_campaign(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn send_newsletter_campaign(
        &self,
        request: Request<SendNewsletterCampaignRequest>,
    ) -> Result<Response<NewsletterCampaignsResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;
        // Manages its own (short) transactions around a slow external-call loop — see
        // procedures::newsletter_campaign's module doc for why this isn't wrapped in one txn.
        procedures::newsletter_campaign::send_newsletter_campaign(db, request).await
    }

    // Sizes Services
    async fn create_size(
        &self,
        request: Request<CreateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::sizes::create_size(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_size(
        &self,
        request: Request<SearchSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::sizes::search_size(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_size(
        &self,
        request: Request<UpdateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::sizes::update_size(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_size(
        &self,
        request: Request<DeleteSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::sizes::delete_size(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Fabrics Services
    async fn create_fabric(
        &self,
        request: Request<CreateFabricRequest>,
    ) -> Result<Response<FabricsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::fabrics::create_fabric(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Weaves Services
    async fn create_weave(
        &self,
        request: Request<CreateWeaveRequest>,
    ) -> Result<Response<WeavesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::weaves::create_weave(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_weave(
        &self,
        request: Request<SearchWeaveRequest>,
    ) -> Result<Response<WeavesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::weaves::search_weave(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_weave(
        &self,
        request: Request<UpdateWeaveRequest>,
    ) -> Result<Response<WeavesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::weaves::update_weave(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_weave(
        &self,
        request: Request<DeleteWeaveRequest>,
    ) -> Result<Response<WeavesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::weaves::delete_weave(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Occasions Services
    async fn create_occasion(
        &self,
        request: Request<CreateOccasionRequest>,
    ) -> Result<Response<OccasionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::occasions::create_occasion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_occasion(
        &self,
        request: Request<SearchOccasionRequest>,
    ) -> Result<Response<OccasionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::occasions::search_occasion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_occasion(
        &self,
        request: Request<UpdateOccasionRequest>,
    ) -> Result<Response<OccasionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::occasions::update_occasion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_occasion(
        &self,
        request: Request<DeleteOccasionRequest>,
    ) -> Result<Response<OccasionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::occasions::delete_occasion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_fabric(
        &self,
        request: Request<SearchFabricRequest>,
    ) -> Result<Response<FabricsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::fabrics::search_fabric(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_fabric(
        &self,
        request: Request<UpdateFabricRequest>,
    ) -> Result<Response<FabricsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::fabrics::update_fabric(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_fabric(
        &self,
        request: Request<DeleteFabricRequest>,
    ) -> Result<Response<FabricsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::fabrics::delete_fabric(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Colors Services
    async fn create_color(
        &self,
        request: Request<CreateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::colors::create_color(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_color(
        &self,
        request: Request<SearchColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::colors::search_color(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_color(
        &self,
        request: Request<UpdateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::colors::update_color(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_color(
        &self,
        request: Request<DeleteColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::colors::delete_color(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductMoodMapping Services
    async fn create_product_mood_mapping(
        &self,
        request: Request<CreateProductMoodMappingRequest>,
    ) -> Result<Response<ProductMoodMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_mood_mappings::create_product_mood_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_mood_mapping(
        &self,
        request: Request<SearchProductMoodMappingRequest>,
    ) -> Result<Response<ProductMoodMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_mood_mappings::search_product_mood_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_mood_mapping(
        &self,
        request: Request<DeleteProductMoodMappingRequest>,
    ) -> Result<Response<ProductMoodMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_mood_mappings::delete_product_mood_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductVariant Services
    async fn create_product_variant(
        &self,
        request: Request<CreateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_variants::create_product_variant(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_variant(
        &self,
        request: Request<SearchProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_variants::search_product_variant(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product_variant(
        &self,
        request: Request<UpdateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_variants::update_product_variant(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_variant(
        &self,
        request: Request<DeleteProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_variants::delete_product_variant(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // EventLogs Services
    async fn create_event_log(
        &self,
        request: Request<CreateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::event_logs::create_event_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_event_log(
        &self,
        request: Request<SearchEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::event_logs::search_event_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_event_log(
        &self,
        request: Request<UpdateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::event_logs::update_event_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_event_log(
        &self,
        request: Request<DeleteEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::event_logs::delete_event_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // UserActivity Services
    async fn create_user_activity(
        &self,
        request: Request<CreateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_activities::create_user_activity(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_user_activity(
        &self,
        request: Request<SearchUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_activities::search_user_activity(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_user_activity(
        &self,
        request: Request<UpdateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_activities::update_user_activity(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_user_activity(
        &self,
        request: Request<DeleteUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_activities::delete_user_activity(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // InventoryLog Services
    async fn create_inventory_log(
        &self,
        request: Request<CreateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory_logs::create_inventory_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_inventory_log(
        &self,
        request: Request<SearchInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory_logs::search_inventory_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_inventory_log(
        &self,
        request: Request<UpdateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory_logs::update_inventory_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_inventory_log(
        &self,
        request: Request<DeleteInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory_logs::delete_inventory_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_payment_intent(
        &self,
        request: Request<CreatePaymentIntentRequest>,
    ) -> Result<Response<PaymentIntentsResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?;

        let mut req = request.into_inner();
        let caller_supplied_order_id = req
            .razorpay_order_id
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty());
        if !caller_supplied_order_id {
            // Resolve the Razorpay order (external HTTP call, up to 15s) before opening the
            // DB transaction below, so the round-trip doesn't hold a pooled connection idle.
            let (razorpay_order_id, amount_paise, currency) =
                handlers::payment_intents::resolve_server_created_razorpay_order(db, req.order_id)
                    .await?;
            req.razorpay_order_id = Some(razorpay_order_id);
            req.amount_paise = amount_paise;
            req.currency = Some(currency);
        }

        let txn = db.begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::create_payment_intent(&txn, Request::new(req)).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn capture_payment(
        &self,
        request: Request<CapturePaymentRequest>,
    ) -> Result<Response<PaymentIntentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::capture_payment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_payment_intent(
        &self,
        request: Request<GetPaymentIntentRequest>,
    ) -> Result<Response<PaymentIntentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::get_payment_intent(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn verify_razorpay_payment(
        &self,
        request: Request<VerifyRazorpayPaymentRequest>,
    ) -> Result<Response<VerifyRazorpayPaymentResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::verify_razorpay_payment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_shipment(
        &self,
        request: Request<CreateShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipments::create_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_shipment(
        &self,
        request: Request<UpdateShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipments::update_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_shipment(
        &self,
        request: Request<GetShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipments::get_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn sync_order_shipments_from_shiprocket(
        &self,
        request: Request<SyncOrderShipmentsFromShiprocketRequest>,
    ) -> Result<Response<SyncOrderShipmentsFromShiprocketResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("database not initialized"))?;
        let order_id = request.into_inner().order_id;
        handlers::shipments::sync_order_shipments_from_shiprocket(db, order_id).await
    }

    async fn validate_coupon(
        &self,
        request: Request<ValidateCouponRequest>,
    ) -> Result<Response<CouponsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::validate_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn apply_coupon(
        &self,
        request: Request<ApplyCouponRequest>,
    ) -> Result<Response<CouponsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::apply_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn list_active_coupons(
        &self,
        request: Request<ListActiveCouponsRequest>,
    ) -> Result<Response<PublicCouponsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::list_active_coupons(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_coupon(
        &self,
        request: Request<CreateCouponRequest>,
    ) -> Result<Response<CouponsAdminResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::create_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_coupon(
        &self,
        request: Request<UpdateCouponRequest>,
    ) -> Result<Response<CouponsAdminResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::update_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_coupon_admin(
        &self,
        request: Request<SearchCouponAdminRequest>,
    ) -> Result<Response<CouponsAdminResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::search_coupon_admin(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_coupon_admin(
        &self,
        request: Request<DeleteCouponAdminRequest>,
    ) -> Result<Response<CouponsAdminResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::coupons::delete_coupon_admin(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_order_event(
        &self,
        request: Request<CreateOrderEventRequest>,
    ) -> Result<Response<OrderEventsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_events::create_order_event(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_order_events(
        &self,
        request: Request<GetOrderEventsRequest>,
    ) -> Result<Response<OrderEventsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_events::get_order_events(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_order_events(
        &self,
        request: Request<SearchOrderEventsRequest>,
    ) -> Result<Response<OrderEventsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::order_events::search_order_events(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_refund(
        &self,
        request: Request<CreateRefundRequest>,
    ) -> Result<Response<RefundsResponse>, Status> {
        let req = request.into_inner();
        let mut attempt: u32 = 0;
        loop {
            let txn = self
                .db
                .as_ref()
                .ok_or_else(|| Status::unavailable("database not initialized"))?
                .begin()
                .await
                .map_err(map_db_error_to_status)?;
            let result =
                match handlers::refunds::create_refund(&txn, Request::new(req.clone())).await {
                    Ok(res) => txn
                        .commit()
                        .await
                        .map_err(map_db_error_to_status)
                        .map(|_| res),
                    Err(status) => Err(status),
                };
            match result {
                Err(status) if attempt < DEADLOCK_MAX_RETRIES && is_deadlock_status(&status) => {
                    attempt += 1;
                    tracing::warn!(attempt, "create_refund: retrying after InnoDB deadlock");
                    continue;
                }
                other => return other,
            }
        }
    }

    async fn get_refunds(
        &self,
        request: Request<GetRefundsRequest>,
    ) -> Result<Response<RefundsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::refunds::get_refunds(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn resolve_needs_review(
        &self,
        request: Request<ResolveNeedsReviewRequest>,
    ) -> Result<Response<ResolveNeedsReviewResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::resolve_needs_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn resolve_refund_attempt_needs_review(
        &self,
        request: Request<ResolveRefundAttemptNeedsReviewRequest>,
    ) -> Result<Response<ResolveRefundAttemptNeedsReviewResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::refunds::resolve_refund_attempt_needs_review(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_refund_attempts(
        &self,
        request: Request<SearchRefundAttemptsRequest>,
    ) -> Result<Response<RefundAttemptsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::refunds::search_refund_attempts(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_order_stats(
        &self,
        request: Request<GetOrderStatsRequest>,
    ) -> Result<Response<GetOrderStatsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::get_order_stats(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn ingest_webhook(
        &self,
        request: Request<IngestWebhookRequest>,
    ) -> Result<Response<WebhookEventsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::webhooks::ingest_webhook(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_presigned_upload_url(
        &self,
        request: Request<GetPresignedUploadUrlRequest>,
    ) -> Result<Response<PresignedUploadUrlResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("Database not initialized"))?;
        handlers::product_images::get_presigned_upload_url(db, request).await
    }

    async fn confirm_image_upload(
        &self,
        request: Request<ConfirmImageUploadRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::confirm_image_upload(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn sync_product_images(
        &self,
        request: Request<SyncProductImagesRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("database not initialized"))?
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::sync_product_images(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn readiness(
        &self,
        _request: Request<ReadinessRequest>,
    ) -> Result<Response<ReadinessResponse>, Status> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| Status::unavailable("Database not initialized"))?;
        db.ping()
            .await
            .map_err(|e| Status::unavailable(format!("DB ping failed: {}", e)))?;
        Ok(Response::new(ReadinessResponse {
            ok: true,
            error: None,
        }))
    }
}

#[cfg(test)]
mod grpc_auth_tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use tonic::{metadata::MetadataValue, Code};

    const AUTH_ENV_KEYS: &[&str] = &[
        "APP_ENV",
        "RUST_ENV",
        "NODE_ENV",
        "STRICT_STARTUP_VALIDATION",
        "GRPC_AUTH_TOKEN",
    ];

    fn auth_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct AuthEnvGuard {
        _lock: MutexGuard<'static, ()>,
        originals: Vec<(&'static str, Option<String>)>,
    }

    impl AuthEnvGuard {
        fn new() -> Self {
            let lock = auth_env_lock()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let originals = AUTH_ENV_KEYS
                .iter()
                .copied()
                .map(|key| (key, std::env::var(key).ok()))
                .collect();
            Self {
                _lock: lock,
                originals,
            }
        }

        fn set(&self, key: &str, value: &str) {
            std::env::set_var(key, value);
        }

        fn remove(&self, key: &str) {
            std::env::remove_var(key);
        }
    }

    impl Drop for AuthEnvGuard {
        fn drop(&mut self) {
            for (key, previous) in &self.originals {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    fn request_with_bearer(token: &str) -> Request<()> {
        let mut req = Request::new(());
        req.metadata_mut().insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {token}")).expect("valid auth metadata"),
        );
        req
    }

    #[test]
    fn is_production_env_treats_unrecognized_value_as_production() {
        let env = AuthEnvGuard::new();
        env.remove("RUST_ENV");
        env.remove("NODE_ENV");
        for typo in ["staging", "prod", "Production", "garbage"] {
            env.set("APP_ENV", typo);
            assert!(
                is_production_env(),
                "APP_ENV={typo} should be treated as production-like (fail closed)"
            );
        }
    }

    #[test]
    fn is_production_env_allows_known_development_aliases_and_unset() {
        let env = AuthEnvGuard::new();
        env.remove("APP_ENV");
        env.remove("RUST_ENV");
        env.remove("NODE_ENV");
        assert!(
            !is_production_env(),
            "unset env vars should not be production"
        );

        for safe_value in ["development", "dev", "local", "test", "Development"] {
            env.set("APP_ENV", safe_value);
            assert!(
                !is_production_env(),
                "APP_ENV={safe_value} should be treated as non-production"
            );
        }
    }

    #[test]
    fn strict_mode_rejects_missing_authorization_header() {
        let env = AuthEnvGuard::new();
        env.set("APP_ENV", "production");
        env.remove("STRICT_STARTUP_VALIDATION");
        env.set("GRPC_AUTH_TOKEN", "expected_token");

        let err = check_auth(Request::new(())).expect_err("missing header must be rejected");
        assert_eq!(err.code(), Code::Unauthenticated);
        assert_eq!(err.message(), "Missing authorization header");
    }

    #[test]
    fn strict_mode_rejects_wrong_authorization_token() {
        let env = AuthEnvGuard::new();
        env.set("APP_ENV", "production");
        env.remove("STRICT_STARTUP_VALIDATION");
        env.set("GRPC_AUTH_TOKEN", "expected_token");

        let err = check_auth(request_with_bearer("wrong_token"))
            .expect_err("wrong token must be rejected");
        assert_eq!(err.code(), Code::Unauthenticated);
        assert_eq!(err.message(), "Invalid authorization token");
    }

    #[test]
    fn strict_mode_accepts_correct_authorization_token() {
        let env = AuthEnvGuard::new();
        env.set("APP_ENV", "production");
        env.remove("STRICT_STARTUP_VALIDATION");
        env.set("GRPC_AUTH_TOKEN", "expected_token");

        let result = check_auth(request_with_bearer("expected_token"));
        assert!(result.is_ok(), "expected token should be accepted");
    }

    #[test]
    fn strict_mode_rejects_when_grpc_auth_token_is_not_configured() {
        let env = AuthEnvGuard::new();
        env.set("STRICT_STARTUP_VALIDATION", "true");
        env.set("APP_ENV", "development");
        env.remove("GRPC_AUTH_TOKEN");

        let err = check_auth(Request::new(()))
            .expect_err("strict mode must reject when GRPC_AUTH_TOKEN is not configured");
        assert_eq!(err.code(), Code::Unauthenticated);
        assert_eq!(err.message(), "gRPC authentication is not configured");
    }

    #[test]
    fn production_mode_rejects_missing_token_even_when_strict_override_is_false() {
        let env = AuthEnvGuard::new();
        env.set("APP_ENV", "production");
        env.set("STRICT_STARTUP_VALIDATION", "false");
        env.remove("GRPC_AUTH_TOKEN");

        let err = check_auth(Request::new(()))
            .expect_err("production mode must reject when GRPC_AUTH_TOKEN is not configured");
        assert_eq!(err.code(), Code::Unauthenticated);
        assert_eq!(err.message(), "gRPC authentication is not configured");
    }

    #[test]
    fn non_strict_mode_allows_missing_grpc_auth_token() {
        let env = AuthEnvGuard::new();
        env.set("APP_ENV", "development");
        env.set("STRICT_STARTUP_VALIDATION", "false");
        env.remove("GRPC_AUTH_TOKEN");

        let result = check_auth(Request::new(()));
        assert!(
            result.is_ok(),
            "non-strict mode should preserve relaxed behavior for missing token"
        );
    }
}

#[cfg(test)]
mod readiness_tests {
    use super::*;
    use proto::proto::core::ReadinessRequest;
    use sea_orm::{DatabaseBackend, MockDatabase};
    use tonic::Request;

    #[tokio::test]
    async fn test_readiness_returns_ok_when_db_ping_succeeds() {
        let db = MockDatabase::new(DatabaseBackend::MySql).into_connection();
        let service = MyGRPCServices {
            db: Some(std::sync::Arc::new(db)),
            session_manager: None,
        };
        let req = Request::new(ReadinessRequest {});
        let result = service.readiness(req).await;
        assert!(
            result.is_ok(),
            "readiness should succeed with mock db: {:?}",
            result.err()
        );
        let res = result.unwrap().into_inner();
        assert!(res.ok);
        assert!(res.error.is_none());
    }

    #[tokio::test]
    async fn test_readiness_returns_unavailable_when_db_not_initialized() {
        let service = MyGRPCServices {
            db: None,
            session_manager: None,
        };
        let req = Request::new(ReadinessRequest {});
        let result = service.readiness(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unavailable);
    }

    /// Guards against a regression back to `self.db.as_ref().unwrap()`: every RPC
    /// method should return a clean `Status::unavailable` (like `readiness()` does)
    /// rather than panicking the handling task if `db` is ever `None` when a request
    /// comes in. Spot-checks one representative RPC; the same `self.db.as_ref()`
    /// pattern is used identically across all ~150 methods in this file.
    #[tokio::test]
    async fn test_rpc_call_returns_unavailable_not_panic_when_db_not_initialized() {
        use proto::proto::core::EnqueueAbandonedCartRequest;

        let service = MyGRPCServices {
            db: None,
            session_manager: None,
        };
        let req = Request::new(EnqueueAbandonedCartRequest { delay_hours: None });
        let result = service.enqueue_abandoned_cart(req).await;
        assert!(
            result.is_err(),
            "expected Err(Status::unavailable), got Ok (or a panic would have aborted this test)"
        );
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unavailable);
    }
}
