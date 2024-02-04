use core_db_entities::{get_db, CoreDatabaseConnection};
use handlers::db_errors::map_db_error_to_status;
use proto::proto::core::{
    grpc_services_server::GrpcServices, AddProductImageRequest, AddWishlistItemRequest,
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
}

pub fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    req.metadata().get("authorization");

    Ok(req)
}

impl MyGRPCServices {
    pub async fn init(&mut self) {
        let db = get_db().await.unwrap();
        self.db = Some(db);
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
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::state_city_mapping::create_state_city_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_state_city_mapping(
        &self,
        request: Request<UpdateStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::state_city_mapping::update_state_city_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_state_city_mapping(
        &self,
        request: Request<DeleteStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::state_city_mapping::delete_state_city_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_state_city_mapping(
        &self,
        request: Request<SearchStateCityMappingRequest>,
    ) -> Result<Response<StateCityMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::state_city_mapping::search_state_city_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // CountryState Mapping
    async fn create_country_state_mapping(
        &self,
        request: Request<CreateCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::country_state_mapping::create_country_state_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_country_state_mapping(
        &self,
        request: Request<UpdateCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::country_state_mapping::update_country_state_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_country_state_mapping(
        &self,
        request: Request<DeleteCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::country_state_mapping::delete_country_state_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_country_state_mapping(
        &self,
        request: Request<SearchCountryStateMappingRequest>,
    ) -> Result<Response<CountryStateMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::country_state_mapping::search_country_state_mapping(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Shipping Address Service
    async fn create_shipping_address(
        &self,
        request: Request<CreateShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::shipping_address::create_shipping_address(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_shipping_address(
        &self,
        request: Request<UpdateShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::shipping_address::update_shipping_address(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_shipping_address(
        &self,
        request: Request<DeleteShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::shipping_address::delete_shipping_address(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn get_shipping_address(
        &self,
        request: Request<GetShippingAddressRequest>,
    ) -> Result<Response<ShippingAddressesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::shipping_address::get_shipping_address(&txn, request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
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
        _request: Request<CreateProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::create_product_color_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;

        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_color_mapping(
        &self,
        _request: Request<SearchProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::search_product_color_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;

        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_color_mapping(
        &self,
        _request: Request<DeleteProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::delete_product_color_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;

        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
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
        _request: Request<CreateProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::create_product_size_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_size_mapping(
        &self,
        _request: Request<SearchProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::search_product_size_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_size_mapping(
        &self,
        _request: Request<DeleteProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::delete_product_size_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // User Services
    async fn create_user(
        &self,
        _request: Request<CreateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::create_user(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user(
        &self,
        _request: Request<SearchUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::search_user(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::update_user(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user(
        &self,
        _request: Request<DeleteUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::delete_user(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
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
        _request: Request<CreateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::create_review(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_review(
        &self,
        _request: Request<SearchReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::search_review(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_review(
        &self,
        _request: Request<UpdateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::update_review(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_review(
        &self,
        _request: Request<DeleteReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::delete_review(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductImages Services
    async fn add_product_image(
        &self,
        request: Request<AddProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::product_images::add_product_image(&txn, request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

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
        _request: Request<CreateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::create_supplier(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_supplier(
        &self,
        _request: Request<SearchSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::search_supplier(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_supplier(
        &self,
        _request: Request<UpdateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::update_supplier(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_supplier(
        &self,
        _request: Request<DeleteSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::delete_supplier(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Inventory Services
    async fn create_inventory_item(
        &self,
        _request: Request<CreateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::create_inventory_item(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_inventory_item(
        &self,
        _request: Request<SearchInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::search_inventory_item(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_inventory_item(
        &self,
        _request: Request<UpdateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::update_inventory_item(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_inventory_item(
        &self,
        _request: Request<DeleteInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::delete_inventory_item(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
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
        _request: Request<CreateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::create_product_attribute(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_attribute(
        &self,
        _request: Request<SearchProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::search_product_attribute(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_attribute(
        &self,
        _request: Request<UpdateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::update_product_attribute(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_attribute(
        &self,
        _request: Request<DeleteProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::delete_product_attribute(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Discounts Services
    async fn create_discount(
        &self,
        _request: Request<CreateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::create_discount(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_discount(
        &self,
        _request: Request<SearchDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::search_discount(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_discount(
        &self,
        _request: Request<UpdateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::update_discount(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_discount(
        &self,
        _request: Request<DeleteDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::delete_discount(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ShippingMethods Services
    async fn create_shipping_method(
        &self,
        _request: Request<CreateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::create_shipping_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_shipping_method(
        &self,
        _request: Request<SearchShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::search_shipping_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_shipping_method(
        &self,
        _request: Request<UpdateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::update_shipping_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_shipping_method(
        &self,
        _request: Request<DeleteShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::delete_shipping_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // UserRole Services
    async fn create_user_role(
        &self,
        _request: Request<CreateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::create_user_role(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_role(
        &self,
        _request: Request<SearchUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::search_user_role(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user_role(
        &self,
        _request: Request<UpdateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::update_user_role(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_role(
        &self,
        _request: Request<DeleteUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::delete_user_role(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Transactions Services
    async fn create_transaction(
        &self,
        _request: Request<CreateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::create_transaction(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_transaction(
        &self,
        _request: Request<SearchTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::search_transaction(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_transaction(
        &self,
        _request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::update_transaction(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_transaction(
        &self,
        _request: Request<DeleteTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::delete_transaction(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // NewsletterSubscribers Services
    async fn create_newsletter_subscriber(
        &self,
        _request: Request<CreateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::create_newsletter_subscriber(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_newsletter_subscriber(
        &self,
        _request: Request<SearchNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::search_newsletter_subscriber(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_newsletter_subscriber(
        &self,
        _request: Request<UpdateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::update_newsletter_subscriber(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_newsletter_subscriber(
        &self,
        _request: Request<DeleteNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::delete_newsletter_subscriber(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductRatings Services
    async fn create_product_rating(
        &self,
        _request: Request<CreateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::create_product_rating(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_rating(
        &self,
        _request: Request<SearchProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::search_product_rating(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_rating(
        &self,
        _request: Request<UpdateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::update_product_rating(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_rating(
        &self,
        _request: Request<DeleteProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::delete_product_rating(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Sizes Services
    async fn create_size(
        &self,
        _request: Request<CreateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::create_size(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_size(
        &self,
        _request: Request<SearchSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::search_size(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_size(
        &self,
        _request: Request<UpdateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::update_size(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_size(
        &self,
        _request: Request<DeleteSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::delete_size(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Colors Services
    async fn create_color(
        &self,
        _request: Request<CreateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::create_color(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_color(
        &self,
        _request: Request<SearchColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::search_color(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_color(
        &self,
        _request: Request<UpdateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::update_color(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_color(
        &self,
        _request: Request<DeleteColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::delete_color(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductCategoryMapping Services
    async fn create_product_category_mapping(
        &self,
        _request: Request<CreateProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::create_product_category_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_category_mapping(
        &self,
        _request: Request<SearchProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::search_product_category_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_category_mapping(
        &self,
        _request: Request<DeleteProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::delete_product_category_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductAttributeMapping Services
    async fn create_product_attribute_mapping(
        &self,
        _request: Request<CreateProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::create_product_attribute_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_attribute_mapping(
        &self,
        _request: Request<SearchProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::search_product_attribute_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_attribute_mapping(
        &self,
        _request: Request<DeleteProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::delete_product_attribute_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // UserRoleMapping Services
    async fn create_user_role_mapping(
        &self,
        _request: Request<CreateUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::create_user_role_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_role_mapping(
        &self,
        _request: Request<SearchUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::search_user_role_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_role_mapping(
        &self,
        _request: Request<DeleteUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::delete_user_role_mapping(
        //     &txn,
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductVariant Services
    async fn create_product_variant(
        &self,
        _request: Request<CreateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::create_product_variant(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_variant(
        &self,
        _request: Request<SearchProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::search_product_variant(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_variant(
        &self,
        _request: Request<UpdateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::update_product_variant(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_variant(
        &self,
        _request: Request<DeleteProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::delete_product_variant(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // EventLogs Services
    async fn create_event_log(
        &self,
        _request: Request<CreateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::create_event_log(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_event_log(
        &self,
        _request: Request<SearchEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::search_event_log(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_event_log(
        &self,
        _request: Request<UpdateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::update_event_log(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_event_log(
        &self,
        _request: Request<DeleteEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::delete_event_log(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // UserActivity Services
    async fn create_user_activity(
        &self,
        _request: Request<CreateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::create_user_activity(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_activity(
        &self,
        _request: Request<SearchUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::search_user_activity(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user_activity(
        &self,
        _request: Request<UpdateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::update_user_activity(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_activity(
        &self,
        _request: Request<DeleteUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::delete_user_activity(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // InventoryLog Services
    async fn create_inventory_log(
        &self,
        _request: Request<CreateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::create_inventory_log(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_inventory_log(
        &self,
        _request: Request<SearchInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::search_inventory_log(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_inventory_log(
        &self,
        _request: Request<UpdateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::update_inventory_log(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_inventory_log(
        &self,
        _request: Request<DeleteInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::delete_inventory_log(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Promotions Services
    async fn create_promotion(
        &self,
        _request: Request<CreatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::create_promotion(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_promotion(
        &self,
        _request: Request<SearchPromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::search_promotion(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_promotion(
        &self,
        _request: Request<UpdatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::update_promotion(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_promotion(
        &self,
        _request: Request<DeletePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::delete_promotion(&txn, request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ShippingZones Services
    async fn create_shipping_zone(
        &self,
        _request: Request<CreateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::create_shipping_zone(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_shipping_zone(
        &self,
        _request: Request<SearchShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::search_shipping_zone(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_shipping_zone(
        &self,
        _request: Request<UpdateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::update_shipping_zone(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_shipping_zone(
        &self,
        _request: Request<DeleteShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::delete_shipping_zone(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // PaymentMethods Services
    async fn create_payment_method(
        &self,
        _request: Request<CreatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::create_payment_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_payment_method(
        &self,
        _request: Request<SearchPaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::search_payment_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_payment_method(
        &self,
        _request: Request<UpdatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::update_payment_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_payment_method(
        &self,
        _request: Request<DeletePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::delete_payment_method(&txn, request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }
}
