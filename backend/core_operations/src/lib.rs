use core_db_entities::{get_db, CoreDatabaseConnection};
use handlers::db_errors::map_db_error_to_status;
use std::time::Duration;

// Phase 1 additions
pub mod auth;
pub mod services;

use proto::proto::core::{
    grpc_services_server::GrpcServices, AddWishlistItemRequest,
    CartItemsResponse, CategoriesResponse, CitiesResponse, ColorsResponse, CountriesResponse,
    CountryStateMappingsResponse, CreateCartItemRequest, CreateCategoryRequest, CreateCityRequest,
    CreateColorRequest, CreateCountryRequest, CreateCountryStateMappingRequest,
    CreateDiscountRequest, CreateEventLogRequest, CreateInventoryItemRequest,
    CreateInventoryLogRequest, CreateNewsletterSubscriberRequest, CreateOrderDetailsRequest,
    CreateOrderRequest, CreatePaymentMethodRequest, CreateProductAttributeMappingRequest,
    CreateProductAttributeRequest, CreateProductCategoryMappingRequest,
    CreateProductColorMappingRequest, CreateProductRatingRequest, CreateProductRequest,
    CreateProductSizeMappingRequest, CreateProductVariantRequest, CreatePromotionRequest,
    CreateReviewRequest, CreateShippingAddressRequest, CreateShippingMethodRequest,
    CreateShippingZoneRequest, CreateSizeRequest, CreateStateCityMappingRequest,
    CreateStateRequest, CreateSupplierRequest, CreateTransactionRequest, CreateUserActivityRequest,
    CreateUserRequest, CreateUserRoleMappingRequest, CreateUserRoleRequest, DeleteCartItemRequest,
    DeleteCategoryRequest, DeleteCityRequest, DeleteColorRequest, DeleteCountryRequest,
    DeleteCountryStateMappingRequest, DeleteDiscountRequest, DeleteEventLogRequest,
    DeleteInventoryItemRequest, DeleteInventoryLogRequest, DeleteNewsletterSubscriberRequest,
    DeleteOrderRequest, DeletePaymentMethodRequest, DeleteProductAttributeMappingRequest,
    DeleteProductAttributeRequest, DeleteProductCategoryMappingRequest,
    DeleteProductColorMappingRequest, DeleteProductImageRequest, DeleteProductRatingRequest,
    DeleteProductRequest, DeleteProductSizeMappingRequest, DeleteProductVariantRequest,
    DeletePromotionRequest, DeleteReviewRequest, DeleteShippingAddressRequest,
    DeleteShippingMethodRequest, DeleteShippingZoneRequest, DeleteSizeRequest,
    DeleteStateCityMappingRequest, DeleteStateRequest, DeleteSupplierRequest,
    DeleteTransactionRequest, DeleteUserActivityRequest, DeleteUserRequest,
    DeleteUserRoleMappingRequest, DeleteUserRoleRequest, DeleteWishlistItemRequest,
    DiscountsResponse, EventLogsResponse, GetCartItemsRequest, GetProductsByIdRequest,
    GetShippingAddressRequest, InventoryItemsResponse, InventoryLogsResponse,
    NewsletterSubscribersResponse, OrderDetailsResponse, OrdersResponse, PaymentMethodsResponse,
    PlaceOrderRequest, ProductAttributeMappingsResponse, ProductAttributesResponse,
    ProductCategoryMappingsResponse, ProductColorMappingsResponse, ProductImagesResponse,
    ProductRatingsResponse, ProductSizeMappingsResponse, ProductVariantsResponse, ProductsResponse,
    PromotionsResponse, ReviewsResponse, SearchCategoryRequest, SearchCityRequest,
    SearchColorRequest, SearchCountryRequest, SearchCountryStateMappingRequest,
    SearchDiscountRequest, SearchEventLogRequest, SearchInventoryItemRequest,
    SearchInventoryLogRequest, SearchNewsletterSubscriberRequest, SearchOrderDetailRequest,
    SearchOrderRequest, SearchPaymentMethodRequest, SearchProductAttributeMappingRequest,
    SearchProductAttributeRequest, SearchProductCategoryMappingRequest,
    SearchProductColorMappingRequest, SearchProductImageRequest, SearchProductRatingRequest,
    SearchProductRequest, SearchProductSizeMappingRequest, SearchProductVariantRequest,
    SearchPromotionRequest, SearchReviewRequest, SearchShippingMethodRequest,
    SearchShippingZoneRequest, SearchSizeRequest, SearchStateCityMappingRequest,
    SearchStateRequest, SearchSupplierRequest, SearchTransactionRequest, SearchUserActivityRequest,
    SearchUserRequest, SearchUserRoleMappingRequest, SearchUserRoleRequest,
    SearchWishlistItemRequest, ShippingAddressesResponse, ShippingMethodsResponse,
    ShippingZonesResponse, SizesResponse, StateCityMappingsResponse, StatesResponse,
    SuppliersResponse, TransactionsResponse, UpdateCartItemRequest, UpdateCategoryRequest,
    UpdateColorRequest, UpdateCountryStateMappingRequest, UpdateDiscountRequest,
    UpdateEventLogRequest, UpdateInventoryItemRequest, UpdateInventoryLogRequest,
    UpdateNewsletterSubscriberRequest, UpdateOrderDetailRequest, UpdateOrderRequest,
    CapturePaymentRequest, GetPaymentIntentRequest, CreatePaymentIntentRequest,
    PaymentIntentsResponse,
    CreateShipmentRequest, UpdateShipmentRequest, GetShipmentRequest, ShipmentsResponse,
    ValidateCouponRequest, ApplyCouponRequest, CouponsResponse,
    CreateOrderEventRequest, GetOrderEventsRequest, OrderEventsResponse,
    IngestWebhookRequest, WebhookEventsResponse,
    GetPresignedUploadUrlRequest, PresignedUploadUrlResponse,
    ConfirmImageUploadRequest,
    UpdatePaymentMethodRequest, UpdateProductAttributeRequest, UpdateProductImageRequest,
    UpdateProductRatingRequest, UpdateProductRequest, UpdateProductVariantRequest,
    UpdatePromotionRequest, UpdateReviewRequest, UpdateShippingAddressRequest,
    UpdateShippingMethodRequest, UpdateShippingZoneRequest, UpdateSizeRequest,
    UpdateStateCityMappingRequest, UpdateSupplierRequest, UpdateTransactionRequest,
    UpdateUserActivityRequest, UpdateUserRequest, UpdateUserRoleRequest, UserActivitiesResponse,
    UserRoleMappingsResponse, UserRolesResponse, UsersResponse, WishlistItemsResponse,
};

use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

mod handlers;
mod procedures;

#[derive(Default, Debug)]
pub struct MyGRPCServices {
    db: Option<CoreDatabaseConnection>,
    session_manager: Option<auth::session::SessionManager>,
}

/// gRPC interceptor that enforces bearer-token auth when `GRPC_AUTH_TOKEN` is set.
///
/// - If `GRPC_AUTH_TOKEN` env var is absent or empty → pass-through (dev/local mode).
/// - All calls must supply `authorization: Bearer <GRPC_AUTH_TOKEN>`.
/// - Auth failures are logged for audit purposes.
pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let expected_token = match std::env::var("GRPC_AUTH_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return Ok(req), // Token not configured — dev/local mode, allow all
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
    pub async fn init(&mut self) {
        let db = get_db().await.unwrap();
        self.db = Some(db);

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            match auth::session::SessionManager::new(&redis_url, Duration::from_secs(86400)) {
                Ok(sm) => self.session_manager = Some(sm),
                Err(e) => log::warn!("Redis session manager not available: {}", e),
            }
        }
    }
}

#[tonic::async_trait]
impl GrpcServices for MyGRPCServices {
    // Country Services
    async fn create_country(
        &self,
        request: Request<CreateCountryRequest>,
    ) -> Result<Response<CountriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::country::create_country(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_country(
        &self,
        request: Request<SearchCountryRequest>,
    ) -> Result<Response<CountriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::country::search_country(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_country(
        &self,
        request: Request<DeleteCountryRequest>,
    ) -> Result<Response<CountriesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::country::delete_country(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // State Service
    async fn create_state(
        &self,
        request: Request<CreateStateRequest>,
    ) -> Result<Response<StatesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state::create_state(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_state(
        &self,
        request: Request<SearchStateRequest>,
    ) -> Result<Response<StatesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state::search_state(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_state(
        &self,
        request: Request<DeleteStateRequest>,
    ) -> Result<Response<StatesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state::delete_state(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // City Service
    async fn create_city(
        &self,
        request: Request<CreateCityRequest>,
    ) -> Result<Response<CitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::city::create_city(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_city(
        &self,
        request: Request<SearchCityRequest>,
    ) -> Result<Response<CitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::city::search_city(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_city(
        &self,
        request: Request<DeleteCityRequest>,
    ) -> Result<Response<CitiesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::city::delete_city(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // CityState Mapping Service
    async fn create_state_city_mapping(
        &self,
        request: Request<CreateStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state_city_mapping::create_state_city_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_state_city_mapping(
        &self,
        request: Request<UpdateStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state_city_mapping::update_state_city_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_state_city_mapping(
        &self,
        request: Request<DeleteStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state_city_mapping::delete_state_city_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_state_city_mapping(
        &self,
        request: Request<SearchStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::state_city_mapping::search_state_city_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // CountryState Mapping
    async fn create_country_state_mapping(
        &self,
        request: Request<CreateCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::country_state_mapping::create_country_state_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_country_state_mapping(
        &self,
        request: Request<UpdateCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::country_state_mapping::update_country_state_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_country_state_mapping(
        &self,
        request: Request<DeleteCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::country_state_mapping::delete_country_state_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_country_state_mapping(
        &self,
        request: Request<SearchCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::country_state_mapping::search_country_state_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Shipping Address Service
    async fn create_shipping_address(
        &self,
        request: Request<CreateShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::cart::delete_cart_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductColorMapping Services
    async fn create_product_color_mapping(
        &self,
        request: Request<CreateProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;

        let res =
            handlers::product_color_mappings::create_product_color_mapping(&txn, request).await?;

        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_color_mapping(
        &self,
        request: Request<SearchProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;

        let res =
            handlers::product_color_mappings::search_product_color_mapping(&txn, request).await?;

        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_color_mapping(
        &self,
        request: Request<DeleteProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;

        let res =
            handlers::product_color_mappings::delete_product_color_mapping(&txn, request).await?;

        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Product Services
    async fn create_product(
        &self,
        request: Request<CreateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::get_products_by_id(&txn, request).await?;
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::delete_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product(
        &self,
        request: Request<UpdateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::products::update_product(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductSizeMapping Services
    async fn create_product_size_mapping(
        &self,
        request: Request<CreateProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_size_mappings::create_product_size_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_size_mapping(
        &self,
        request: Request<SearchProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_size_mappings::search_product_size_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_size_mapping(
        &self,
        request: Request<DeleteProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_size_mappings::delete_product_size_mapping(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::users::delete_user(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = procedures::orders::place_order(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_order(
        &self,
        request: Request<SearchOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::update_order(&txn, request).await?;
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::delete_order(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::reviews::delete_review(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::delete_product_image(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Suppliers Services
    async fn create_supplier(
        &self,
        request: Request<CreateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::suppliers::create_supplier(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_supplier(
        &self,
        request: Request<SearchSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::suppliers::search_supplier(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_supplier(
        &self,
        request: Request<UpdateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::suppliers::update_supplier(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_supplier(
        &self,
        request: Request<DeleteSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::suppliers::delete_supplier(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::wishlist::delete_wishlist_item(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductAttributes Services
    async fn create_product_attribute(
        &self,
        request: Request<CreateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_attributes::create_product_attribute(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_attribute(
        &self,
        request: Request<SearchProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_attributes::search_product_attribute(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product_attribute(
        &self,
        request: Request<UpdateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_attributes::update_product_attribute(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_attribute(
        &self,
        request: Request<DeleteProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_attributes::delete_product_attribute(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Discounts Services
    async fn create_discount(
        &self,
        request: Request<CreateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::discounts::create_discount(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_discount(
        &self,
        request: Request<SearchDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::discounts::search_discount(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_discount(
        &self,
        request: Request<UpdateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::discounts::update_discount(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_discount(
        &self,
        request: Request<DeleteDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::discounts::delete_discount(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::newsletter_subscribers::delete_newsletter_subscriber(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductRatings Services
    async fn create_product_rating(
        &self,
        request: Request<CreateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_ratings::create_product_rating(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_rating(
        &self,
        request: Request<SearchProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_ratings::search_product_rating(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_product_rating(
        &self,
        request: Request<UpdateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_ratings::update_product_rating(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_rating(
        &self,
        request: Request<DeleteProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_ratings::delete_product_rating(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Sizes Services
    async fn create_size(
        &self,
        request: Request<CreateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::sizes::delete_size(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::colors::delete_color(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductCategoryMapping Services
    async fn create_product_category_mapping(
        &self,
        request: Request<CreateProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_category_mappings::create_product_category_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_category_mapping(
        &self,
        request: Request<SearchProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_category_mappings::search_product_category_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_category_mapping(
        &self,
        request: Request<DeleteProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_category_mappings::delete_product_category_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductAttributeMapping Services
    async fn create_product_attribute_mapping(
        &self,
        request: Request<CreateProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_attribute_mappings::create_product_attribute_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_product_attribute_mapping(
        &self,
        request: Request<SearchProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_attribute_mappings::search_product_attribute_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_product_attribute_mapping(
        &self,
        request: Request<DeleteProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res =
            handlers::product_attribute_mappings::delete_product_attribute_mapping(&txn, request)
                .await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // UserRoleMapping Services
    async fn create_user_role_mapping(
        &self,
        request: Request<CreateUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_role_mappings::create_user_role_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_user_role_mapping(
        &self,
        request: Request<SearchUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_role_mappings::search_user_role_mapping(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_user_role_mapping(
        &self,
        request: Request<DeleteUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::user_role_mappings::delete_user_role_mapping(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::inventory_logs::delete_inventory_log(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // Promotions Services
    async fn create_promotion(
        &self,
        request: Request<CreatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::promotions::create_promotion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_promotion(
        &self,
        request: Request<SearchPromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::promotions::search_promotion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_promotion(
        &self,
        request: Request<UpdatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::promotions::update_promotion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_promotion(
        &self,
        request: Request<DeletePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::promotions::delete_promotion(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ShippingZones Services
    async fn create_shipping_zone(
        &self,
        request: Request<CreateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_zones::create_shipping_zone(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_shipping_zone(
        &self,
        request: Request<SearchShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_zones::search_shipping_zone(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_shipping_zone(
        &self,
        request: Request<UpdateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_zones::update_shipping_zone(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_shipping_zone(
        &self,
        request: Request<DeleteShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::shipping_zones::delete_shipping_zone(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // PaymentMethods Services
    async fn create_payment_method(
        &self,
        request: Request<CreatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_methods::create_payment_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn search_payment_method(
        &self,
        request: Request<SearchPaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_methods::search_payment_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_payment_method(
        &self,
        request: Request<UpdatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_methods::update_payment_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn delete_payment_method(
        &self,
        request: Request<DeletePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_methods::delete_payment_method(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_payment_intent(
        &self,
        request: Request<CreatePaymentIntentRequest>,
    ) -> Result<Response<PaymentIntentsResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::create_payment_intent(&txn, request).await?;
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
            .unwrap()
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
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::payment_intents::get_payment_intent(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_shipment(
        &self,
        request: Request<CreateShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::shipments::create_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn update_shipment(
        &self,
        request: Request<UpdateShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::shipments::update_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_shipment(
        &self,
        request: Request<GetShipmentRequest>,
    ) -> Result<Response<ShipmentsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::shipments::get_shipment(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn validate_coupon(
        &self,
        request: Request<ValidateCouponRequest>,
    ) -> Result<Response<CouponsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::coupons::validate_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn apply_coupon(
        &self,
        request: Request<ApplyCouponRequest>,
    ) -> Result<Response<CouponsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::coupons::apply_coupon(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn create_order_event(
        &self,
        request: Request<CreateOrderEventRequest>,
    ) -> Result<Response<OrderEventsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::order_events::create_order_event(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_order_events(
        &self,
        request: Request<GetOrderEventsRequest>,
    ) -> Result<Response<OrderEventsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::order_events::get_order_events(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn ingest_webhook(
        &self,
        request: Request<IngestWebhookRequest>,
    ) -> Result<Response<WebhookEventsResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::webhooks::ingest_webhook(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    async fn get_presigned_upload_url(
        &self,
        request: Request<GetPresignedUploadUrlRequest>,
    ) -> Result<Response<PresignedUploadUrlResponse>, Status> {
        handlers::product_images::get_presigned_upload_url(request).await
    }

    async fn confirm_image_upload(
        &self,
        request: Request<ConfirmImageUploadRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self.db.as_ref().unwrap().begin().await.map_err(map_db_error_to_status)?;
        let res = handlers::product_images::confirm_image_upload(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }
}
