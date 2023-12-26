use core_db_entities::{get_db, CoreDatabaseConnection};
use handlers::db_errors::map_db_error_to_status;
use proto::proto::core::{
    grpc_services_server::GrpcServices, CartItemsResponse, CategoriesResponse, CategoryResponse,
    ColorResponse, ColorsResponse, CreateCartItemRequest, CreateCategoryRequest,
    CreateColorRequest, CreateDiscountRequest, CreateEventLogRequest, CreateInventoryItemRequest,
    CreateInventoryLogRequest, CreateNewsletterSubscriberRequest, CreateOrderDetailRequest,
    CreateOrderRequest, CreatePaymentMethodRequest, CreateProductAttributeMappingRequest,
    CreateProductAttributeRequest, CreateProductCategoryMappingRequest,
    CreateProductColorMappingRequest, CreateProductImageRequest, CreateProductRatingRequest,
    CreateProductRequest, CreateProductSizeMappingRequest, CreateProductVariantRequest,
    CreatePromotionRequest, CreateReviewRequest, CreateShippingMethodRequest,
    CreateShippingZoneRequest, CreateSizeRequest, CreateSupplierRequest, CreateTransactionRequest,
    CreateUserActivityRequest, CreateUserRequest, CreateUserRoleMappingRequest,
    CreateUserRoleRequest, CreateWishlistItemRequest, DeleteCartItemRequest, DeleteCategoryRequest,
    DeleteColorRequest, DeleteDiscountRequest, DeleteEventLogRequest, DeleteInventoryItemRequest,
    DeleteInventoryLogRequest, DeleteNewsletterSubscriberRequest, DeleteOrderDetailRequest,
    DeleteOrderRequest, DeletePaymentMethodRequest, DeleteProductAttributeMappingRequest,
    DeleteProductAttributeRequest, DeleteProductCategoryMappingRequest,
    DeleteProductColorMappingRequest, DeleteProductImageRequest, DeleteProductRatingRequest,
    DeleteProductRequest, DeleteProductSizeMappingRequest, DeleteProductVariantRequest,
    DeletePromotionRequest, DeleteReviewRequest, DeleteShippingMethodRequest,
    DeleteShippingZoneRequest, DeleteSizeRequest, DeleteSupplierRequest, DeleteTransactionRequest,
    DeleteUserActivityRequest, DeleteUserRequest, DeleteUserRoleMappingRequest,
    DeleteUserRoleRequest, DeleteWishlistItemRequest, DiscountResponse, DiscountsResponse,
    EventLogResponse, EventLogsResponse, GetCartItemsRequest, InventoryItemResponse,
    InventoryItemsResponse, InventoryLogResponse, InventoryLogsResponse,
    NewsletterSubscriberResponse, NewsletterSubscribersResponse, OrderDetailResponse,
    OrderDetailsResponse, OrderResponse, OrdersResponse, PaymentMethodResponse,
    PaymentMethodsResponse, ProductAttributeMappingResponse, ProductAttributeMappingsResponse,
    ProductAttributeResponse, ProductAttributesResponse, ProductCategoryMappingResponse,
    ProductCategoryMappingsResponse, ProductColorMappingResponse, ProductColorMappingsResponse,
    ProductImageResponse, ProductImagesResponse, ProductRatingResponse, ProductRatingsResponse,
    ProductSizeMappingResponse, ProductSizeMappingsResponse, ProductVariantResponse,
    ProductVariantsResponse, ProductsResponse, PromotionResponse, PromotionsResponse,
    ReviewsResponse, SearchCategoryRequest, SearchColorRequest, SearchDiscountRequest,
    SearchEventLogRequest, SearchInventoryItemRequest, SearchInventoryLogRequest,
    SearchNewsletterSubscriberRequest, SearchOrderDetailRequest, SearchOrderRequest,
    SearchPaymentMethodRequest, SearchProductAttributeMappingRequest,
    SearchProductAttributeRequest, SearchProductCategoryMappingRequest,
    SearchProductColorMappingRequest, SearchProductImageRequest, SearchProductRatingRequest,
    SearchProductRequest, SearchProductSizeMappingRequest, SearchProductVariantRequest,
    SearchPromotionRequest, SearchReviewRequest, SearchShippingMethodRequest,
    SearchShippingZoneRequest, SearchSizeRequest, SearchSupplierRequest, SearchTransactionRequest,
    SearchUserActivityRequest, SearchUserRequest, SearchUserRoleMappingRequest,
    SearchUserRoleRequest, SearchWishlistItemRequest, ShippingMethodResponse,
    ShippingMethodsResponse, ShippingZoneResponse, ShippingZonesResponse, SizeResponse,
    SizesResponse, SupplierResponse, SuppliersResponse, TransactionResponse, TransactionsResponse,
    UpdateCartItemRequest, UpdateCategoryRequest, UpdateColorRequest, UpdateDiscountRequest,
    UpdateEventLogRequest, UpdateInventoryItemRequest, UpdateInventoryLogRequest,
    UpdateNewsletterSubscriberRequest, UpdateOrderDetailRequest, UpdateOrderRequest,
    UpdatePaymentMethodRequest, UpdateProductAttributeRequest, UpdateProductImageRequest,
    UpdateProductRatingRequest, UpdateProductRequest, UpdateProductVariantRequest,
    UpdatePromotionRequest, UpdateReviewRequest, UpdateShippingMethodRequest,
    UpdateShippingZoneRequest, UpdateSizeRequest, UpdateSupplierRequest, UpdateTransactionRequest,
    UpdateUserActivityRequest, UpdateUserRequest, UpdateUserRoleRequest, UpdateWishlistItemRequest,
    UserActivitiesResponse, UserActivityResponse, UserResponse, UserRoleMappingResponse,
    UserRoleMappingsResponse, UserRoleResponse, UserRolesResponse, UsersResponse,
    WishlistItemResponse, WishlistItemsResponse,
};

use sea_orm::TransactionTrait;
use tonic::{Request, Response, Status};

mod handlers;

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
        let res = handlers::cart::create_cart_item(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::cart::get_cart_items(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::cart::update_cart_item(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::cart::delete_cart_item(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductColorMapping Services
    async fn create_product_color_mapping(
        &self,
        request: Request<CreateProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::create_product_color_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;

        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_color_mapping(
        &self,
        request: Request<SearchProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::search_product_color_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;

        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_color_mapping(
        &self,
        request: Request<DeleteProductColorMappingRequest>,
    ) -> Result<Response<ProductColorMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;

        // let res = handlers::product_color_mappings::delete_product_color_mapping(
        //     self.db.as_ref().unwrap(),
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
        let res = handlers::products::create_product(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::products::search_product(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::products::delete_product(self.db.as_ref().unwrap(), request).await?;
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
        let res = handlers::products::update_product(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res)
    }

    // ProductSizeMapping Services
    async fn create_product_size_mapping(
        &self,
        request: Request<CreateProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::create_product_size_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_size_mapping(
        &self,
        request: Request<SearchProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::search_product_size_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_size_mapping(
        &self,
        request: Request<DeleteProductSizeMappingRequest>,
    ) -> Result<Response<ProductSizeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_size_mappings::delete_product_size_mapping(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::create_user(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user(
        &self,
        request: Request<SearchUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::search_user(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::update_user(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<UsersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::users::delete_user(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Category Services
    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::categories::create_category(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_category(
        &self,
        request: Request<SearchCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::categories::search_category(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::categories::update_category(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<CategoriesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::categories::delete_category(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Order Services
    // Order Services
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        /* let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::create_order(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res) */
        todo!()
    }

    async fn search_order(
        &self,
        request: Request<SearchOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        /* let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::search_order(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res) */
        todo!()
    }

    async fn update_order(
        &self,
        request: Request<UpdateOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        /* let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::update_order(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res) */
        todo!()
    }

    async fn delete_order(
        &self,
        request: Request<DeleteOrderRequest>,
    ) -> Result<Response<OrdersResponse>, Status> {
        /* let txn = self
            .db
            .as_ref()
            .unwrap()
            .begin()
            .await
            .map_err(map_db_error_to_status)?;
        let res = handlers::orders::delete_order(self.db.as_ref().unwrap(), request).await?;
        txn.commit().await.map_err(map_db_error_to_status)?;
        Ok(res) */
        todo!()
    }

    // OrderDetails Services
    async fn create_order_detail(
        &self,
        request: Request<CreateOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::order_details::create_order_detail(self.db.as_ref().unwrap(), request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_order_detail(
        &self,
        request: Request<SearchOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::order_details::search_order_detail(self.db.as_ref().unwrap(), request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_order_detail(
        &self,
        request: Request<UpdateOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::order_details::update_order_detail(self.db.as_ref().unwrap(), request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_order_detail(
        &self,
        request: Request<DeleteOrderDetailRequest>,
    ) -> Result<Response<OrderDetailsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::order_details::delete_order_detail(self.db.as_ref().unwrap(), request)
        //     .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Reviews Services
    async fn create_review(
        &self,
        request: Request<CreateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::create_review(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_review(
        &self,
        request: Request<SearchReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::search_review(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_review(
        &self,
        request: Request<UpdateReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::update_review(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_review(
        &self,
        request: Request<DeleteReviewRequest>,
    ) -> Result<Response<ReviewsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::reviews::delete_review(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductImages Services
    async fn create_product_image(
        &self,
        request: Request<CreateProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_images::create_product_image(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_image(
        &self,
        request: Request<SearchProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_images::search_product_image(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_image(
        &self,
        request: Request<UpdateProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_images::update_product_image(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_image(
        &self,
        request: Request<DeleteProductImageRequest>,
    ) -> Result<Response<ProductImagesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_images::delete_product_image(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Suppliers Services
    async fn create_supplier(
        &self,
        request: Request<CreateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::create_supplier(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_supplier(
        &self,
        request: Request<SearchSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::search_supplier(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_supplier(
        &self,
        request: Request<UpdateSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::update_supplier(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_supplier(
        &self,
        request: Request<DeleteSupplierRequest>,
    ) -> Result<Response<SuppliersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::suppliers::delete_supplier(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Inventory Services
    async fn create_inventory_item(
        &self,
        request: Request<CreateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::create_inventory_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_inventory_item(
        &self,
        request: Request<SearchInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::search_inventory_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_inventory_item(
        &self,
        request: Request<UpdateInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::update_inventory_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_inventory_item(
        &self,
        request: Request<DeleteInventoryItemRequest>,
    ) -> Result<Response<InventoryItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory::delete_inventory_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Wishlist Services
    async fn create_wishlist_item(
        &self,
        request: Request<CreateWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::wishlist::create_wishlist_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_wishlist_item(
        &self,
        request: Request<SearchWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::wishlist::search_wishlist_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_wishlist_item(
        &self,
        request: Request<UpdateWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::wishlist::update_wishlist_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_wishlist_item(
        &self,
        request: Request<DeleteWishlistItemRequest>,
    ) -> Result<Response<WishlistItemsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::wishlist::delete_wishlist_item(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductAttributes Services
    async fn create_product_attribute(
        &self,
        request: Request<CreateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::create_product_attribute(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_attribute(
        &self,
        request: Request<SearchProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::search_product_attribute(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_attribute(
        &self,
        request: Request<UpdateProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::update_product_attribute(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_attribute(
        &self,
        request: Request<DeleteProductAttributeRequest>,
    ) -> Result<Response<ProductAttributesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attributes::delete_product_attribute(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::create_discount(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_discount(
        &self,
        request: Request<SearchDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::search_discount(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_discount(
        &self,
        request: Request<UpdateDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::update_discount(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_discount(
        &self,
        request: Request<DeleteDiscountRequest>,
    ) -> Result<Response<DiscountsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::discounts::delete_discount(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ShippingMethods Services
    async fn create_shipping_method(
        &self,
        request: Request<CreateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::create_shipping_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_shipping_method(
        &self,
        request: Request<SearchShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::search_shipping_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_shipping_method(
        &self,
        request: Request<UpdateShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::update_shipping_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_shipping_method(
        &self,
        request: Request<DeleteShippingMethodRequest>,
    ) -> Result<Response<ShippingMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_methods::delete_shipping_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // UserRole Services
    async fn create_user_role(
        &self,
        request: Request<CreateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::create_user_role(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_role(
        &self,
        request: Request<SearchUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::search_user_role(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user_role(
        &self,
        request: Request<UpdateUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::update_user_role(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_role(
        &self,
        request: Request<DeleteUserRoleRequest>,
    ) -> Result<Response<UserRolesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_roles::delete_user_role(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Transactions Services
    async fn create_transaction(
        &self,
        request: Request<CreateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::create_transaction(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_transaction(
        &self,
        request: Request<SearchTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::search_transaction(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_transaction(
        &self,
        request: Request<UpdateTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::update_transaction(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_transaction(
        &self,
        request: Request<DeleteTransactionRequest>,
    ) -> Result<Response<TransactionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::transactions::delete_transaction(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // NewsletterSubscribers Services
    async fn create_newsletter_subscriber(
        &self,
        request: Request<CreateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::create_newsletter_subscriber(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_newsletter_subscriber(
        &self,
        request: Request<SearchNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::search_newsletter_subscriber(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_newsletter_subscriber(
        &self,
        request: Request<UpdateNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::update_newsletter_subscriber(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_newsletter_subscriber(
        &self,
        request: Request<DeleteNewsletterSubscriberRequest>,
    ) -> Result<Response<NewsletterSubscribersResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::newsletter_subscribers::delete_newsletter_subscriber(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::create_product_rating(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_rating(
        &self,
        request: Request<SearchProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::search_product_rating(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_rating(
        &self,
        request: Request<UpdateProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::update_product_rating(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_rating(
        &self,
        request: Request<DeleteProductRatingRequest>,
    ) -> Result<Response<ProductRatingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_ratings::delete_product_rating(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Sizes Services
    async fn create_size(
        &self,
        request: Request<CreateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::create_size(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_size(
        &self,
        request: Request<SearchSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::search_size(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_size(
        &self,
        request: Request<UpdateSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::update_size(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_size(
        &self,
        request: Request<DeleteSizeRequest>,
    ) -> Result<Response<SizesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::sizes::delete_size(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Colors Services
    async fn create_color(
        &self,
        request: Request<CreateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::create_color(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_color(
        &self,
        request: Request<SearchColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::search_color(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_color(
        &self,
        request: Request<UpdateColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::update_color(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_color(
        &self,
        request: Request<DeleteColorRequest>,
    ) -> Result<Response<ColorsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::colors::delete_color(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ProductCategoryMapping Services
    async fn create_product_category_mapping(
        &self,
        request: Request<CreateProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::create_product_category_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_category_mapping(
        &self,
        request: Request<SearchProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::search_product_category_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_category_mapping(
        &self,
        request: Request<DeleteProductCategoryMappingRequest>,
    ) -> Result<Response<ProductCategoryMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_category_mappings::delete_product_category_mapping(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::create_product_attribute_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_attribute_mapping(
        &self,
        request: Request<SearchProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::search_product_attribute_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_attribute_mapping(
        &self,
        request: Request<DeleteProductAttributeMappingRequest>,
    ) -> Result<Response<ProductAttributeMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::product_attribute_mappings::delete_product_attribute_mapping(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::create_user_role_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_role_mapping(
        &self,
        request: Request<SearchUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::search_user_role_mapping(
        //     self.db.as_ref().unwrap(),
        //     request,
        // )
        // .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_role_mapping(
        &self,
        request: Request<DeleteUserRoleMappingRequest>,
    ) -> Result<Response<UserRoleMappingsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res = handlers::user_role_mappings::delete_user_role_mapping(
        //     self.db.as_ref().unwrap(),
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
        request: Request<CreateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::create_product_variant(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_product_variant(
        &self,
        request: Request<SearchProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::search_product_variant(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_product_variant(
        &self,
        request: Request<UpdateProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::update_product_variant(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_product_variant(
        &self,
        request: Request<DeleteProductVariantRequest>,
    ) -> Result<Response<ProductVariantsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::product_variants::delete_product_variant(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // EventLogs Services
    async fn create_event_log(
        &self,
        request: Request<CreateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::create_event_log(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_event_log(
        &self,
        request: Request<SearchEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::search_event_log(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_event_log(
        &self,
        request: Request<UpdateEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::update_event_log(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_event_log(
        &self,
        request: Request<DeleteEventLogRequest>,
    ) -> Result<Response<EventLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::event_logs::delete_event_log(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // UserActivity Services
    async fn create_user_activity(
        &self,
        request: Request<CreateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::create_user_activity(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_user_activity(
        &self,
        request: Request<SearchUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::search_user_activity(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_user_activity(
        &self,
        request: Request<UpdateUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::update_user_activity(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_user_activity(
        &self,
        request: Request<DeleteUserActivityRequest>,
    ) -> Result<Response<UserActivitiesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::user_activities::delete_user_activity(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // InventoryLog Services
    async fn create_inventory_log(
        &self,
        request: Request<CreateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::create_inventory_log(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_inventory_log(
        &self,
        request: Request<SearchInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::search_inventory_log(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_inventory_log(
        &self,
        request: Request<UpdateInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::update_inventory_log(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_inventory_log(
        &self,
        request: Request<DeleteInventoryLogRequest>,
    ) -> Result<Response<InventoryLogsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::inventory_logs::delete_inventory_log(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // Promotions Services
    async fn create_promotion(
        &self,
        request: Request<CreatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::create_promotion(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_promotion(
        &self,
        request: Request<SearchPromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::search_promotion(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_promotion(
        &self,
        request: Request<UpdatePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::update_promotion(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_promotion(
        &self,
        request: Request<DeletePromotionRequest>,
    ) -> Result<Response<PromotionsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::promotions::delete_promotion(self.db.as_ref().unwrap(), request).await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // ShippingZones Services
    async fn create_shipping_zone(
        &self,
        request: Request<CreateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::create_shipping_zone(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_shipping_zone(
        &self,
        request: Request<SearchShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::search_shipping_zone(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_shipping_zone(
        &self,
        request: Request<UpdateShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::update_shipping_zone(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_shipping_zone(
        &self,
        request: Request<DeleteShippingZoneRequest>,
    ) -> Result<Response<ShippingZonesResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::shipping_zones::delete_shipping_zone(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    // PaymentMethods Services
    async fn create_payment_method(
        &self,
        request: Request<CreatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::create_payment_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn search_payment_method(
        &self,
        request: Request<SearchPaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::search_payment_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn update_payment_method(
        &self,
        request: Request<UpdatePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::update_payment_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }

    async fn delete_payment_method(
        &self,
        request: Request<DeletePaymentMethodRequest>,
    ) -> Result<Response<PaymentMethodsResponse>, Status> {
        // let txn = self
        //     .db
        //     .as_ref()
        //     .unwrap()
        //     .begin()
        //     .await
        //     .map_err(map_db_error_to_status)?;
        // let res =
        //     handlers::payment_methods::delete_payment_method(self.db.as_ref().unwrap(), request)
        //         .await?;
        // txn.commit().await.map_err(map_db_error_to_status)?;
        // Ok(res)
        todo!()
    }
}
