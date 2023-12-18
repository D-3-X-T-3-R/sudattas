#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCartItemRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub quantity: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCartItemsRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCartItemRequest {
    #[prost(int64, tag = "1")]
    pub cart_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub product_id: i64,
    #[prost(int64, tag = "4")]
    pub quantity: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCartItemRequest {
    #[prost(int64, tag = "1")]
    pub cart_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CartItemResponse {
    #[prost(int64, tag = "1")]
    pub cart_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub product_id: i64,
    #[prost(int64, tag = "4")]
    pub quantity: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CartItemsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<CartItemResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, tag = "3")]
    pub price: f64,
    #[prost(int64, optional, tag = "4")]
    pub stock_quantity: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "5")]
    pub category_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchProductRequest {
    #[prost(string, optional, tag = "1")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "2")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "3")]
    pub starting_price: ::core::option::Option<f64>,
    #[prost(int64, optional, tag = "4")]
    pub stock_quantity: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "5")]
    pub category_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "6")]
    pub ending_price: ::core::option::Option<f64>,
    #[prost(int64, optional, tag = "7")]
    pub product_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, tag = "3")]
    pub price: f64,
    #[prost(int64, optional, tag = "4")]
    pub stock_quantity: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "5")]
    pub category_id: ::core::option::Option<i64>,
    #[prost(int64, tag = "6")]
    pub product_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductResponse {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, tag = "3")]
    pub price: f64,
    #[prost(int64, optional, tag = "4")]
    pub stock_quantity: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "5")]
    pub category_id: ::core::option::Option<i64>,
    #[prost(int64, tag = "6")]
    pub product_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRequest {
    #[prost(string, tag = "1")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub password: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub email: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "4")]
    pub full_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub phone: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, optional, tag = "2")]
    pub username: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub password: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub full_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub phone: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserResponse {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub email: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "4")]
    pub full_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub phone: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "7")]
    pub create_date: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UsersResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<UserResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCategoryRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetCategoryRequest {
    #[prost(int64, tag = "1")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCategoryRequest {
    #[prost(int64, tag = "1")]
    pub category_id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteCategoryRequest {
    #[prost(int64, tag = "1")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CategoryResponse {
    #[prost(int64, tag = "1")]
    pub category_id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CategoriesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<CategoryResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateOrderRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub shipping_address: ::prost::alloc::string::String,
    #[prost(double, tag = "3")]
    pub total_amount: f64,
    #[prost(int64, tag = "4")]
    pub status_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOrderRequest {
    #[prost(int64, tag = "1")]
    pub order_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateOrderRequest {
    #[prost(int64, tag = "1")]
    pub order_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub shipping_address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "4")]
    pub total_amount: ::core::option::Option<f64>,
    #[prost(int64, optional, tag = "5")]
    pub status_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteOrderRequest {
    #[prost(int64, tag = "1")]
    pub order_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderResponse {
    #[prost(int64, tag = "1")]
    pub order_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(string, tag = "3")]
    pub order_date: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub shipping_address: ::prost::alloc::string::String,
    #[prost(double, tag = "5")]
    pub total_amount: f64,
    #[prost(int64, tag = "6")]
    pub status_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrdersResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<OrderResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateOrderDetailRequest {
    #[prost(int64, tag = "1")]
    pub order_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub quantity: i64,
    #[prost(double, tag = "4")]
    pub price: f64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetOrderDetailRequest {
    #[prost(int64, tag = "1")]
    pub order_detail_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateOrderDetailRequest {
    #[prost(int64, tag = "1")]
    pub order_detail_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub order_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub quantity: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "5")]
    pub price: ::core::option::Option<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteOrderDetailRequest {
    #[prost(int64, tag = "1")]
    pub order_detail_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderDetailResponse {
    #[prost(int64, tag = "1")]
    pub order_detail_id: i64,
    #[prost(int64, tag = "2")]
    pub order_id: i64,
    #[prost(int64, tag = "3")]
    pub product_id: i64,
    #[prost(int64, tag = "4")]
    pub quantity: i64,
    #[prost(double, tag = "5")]
    pub price: f64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderDetailsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<OrderDetailResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateReviewRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub rating: i64,
    #[prost(string, tag = "4")]
    pub comment: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetReviewRequest {
    #[prost(int64, tag = "1")]
    pub review_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateReviewRequest {
    #[prost(int64, tag = "1")]
    pub review_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub rating: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "5")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteReviewRequest {
    #[prost(int64, tag = "1")]
    pub review_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReviewResponse {
    #[prost(int64, tag = "1")]
    pub review_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub user_id: i64,
    #[prost(int64, tag = "4")]
    pub rating: i64,
    #[prost(string, tag = "5")]
    pub comment: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReviewsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ReviewResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductImageRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(string, tag = "2")]
    pub image_url: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub alt_text: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductImageRequest {
    #[prost(int64, tag = "1")]
    pub image_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductImageRequest {
    #[prost(int64, tag = "1")]
    pub image_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub image_url: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub alt_text: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductImageRequest {
    #[prost(int64, tag = "1")]
    pub image_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductImageResponse {
    #[prost(int64, tag = "1")]
    pub image_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(string, tag = "3")]
    pub image_url: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub alt_text: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductImagesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductImageResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSupplierRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub contact_info: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub address: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSupplierRequest {
    #[prost(int64, tag = "1")]
    pub supplier_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSupplierRequest {
    #[prost(int64, tag = "1")]
    pub supplier_id: i64,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub contact_info: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSupplierRequest {
    #[prost(int64, tag = "1")]
    pub supplier_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SupplierResponse {
    #[prost(int64, tag = "1")]
    pub supplier_id: i64,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub contact_info: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub address: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SuppliersResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<SupplierResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateInventoryItemRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub quantity_available: i64,
    #[prost(int64, tag = "3")]
    pub reorder_level: i64,
    #[prost(int64, tag = "4")]
    pub supplier_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInventoryItemRequest {
    #[prost(int64, tag = "1")]
    pub inventory_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInventoryItemRequest {
    #[prost(int64, tag = "1")]
    pub inventory_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub quantity_available: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub reorder_level: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "5")]
    pub supplier_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteInventoryItemRequest {
    #[prost(int64, tag = "1")]
    pub inventory_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InventoryItemResponse {
    #[prost(int64, tag = "1")]
    pub inventory_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub quantity_available: i64,
    #[prost(int64, tag = "4")]
    pub reorder_level: i64,
    #[prost(int64, tag = "5")]
    pub supplier_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InventoryItemsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<InventoryItemResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateWishlistItemRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(string, tag = "3")]
    pub date_added: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetWishlistItemRequest {
    #[prost(int64, tag = "1")]
    pub wishlist_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateWishlistItemRequest {
    #[prost(int64, tag = "1")]
    pub wishlist_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "4")]
    pub date_added: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteWishlistItemRequest {
    #[prost(int64, tag = "1")]
    pub wishlist_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WishlistItemResponse {
    #[prost(int64, tag = "1")]
    pub wishlist_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub product_id: i64,
    #[prost(string, tag = "4")]
    pub date_added: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WishlistItemsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<WishlistItemResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductAttributeRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(string, tag = "2")]
    pub attribute_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub attribute_value: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductAttributeRequest {
    #[prost(int64, tag = "1")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductAttributeRequest {
    #[prost(int64, tag = "1")]
    pub attribute_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub attribute_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub attribute_value: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductAttributeRequest {
    #[prost(int64, tag = "1")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductAttributeResponse {
    #[prost(int64, tag = "1")]
    pub attribute_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(string, tag = "3")]
    pub attribute_name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub attribute_value: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductAttributesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductAttributeResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDiscountRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(double, tag = "2")]
    pub discount_percentage: f64,
    #[prost(string, tag = "3")]
    pub start_date: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub end_date: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDiscountRequest {
    #[prost(int64, tag = "1")]
    pub discount_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateDiscountRequest {
    #[prost(int64, tag = "1")]
    pub discount_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "3")]
    pub discount_percentage: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "4")]
    pub start_date: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub end_date: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteDiscountRequest {
    #[prost(int64, tag = "1")]
    pub discount_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscountResponse {
    #[prost(int64, tag = "1")]
    pub discount_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(double, tag = "3")]
    pub discount_percentage: f64,
    #[prost(string, tag = "4")]
    pub start_date: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub end_date: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscountsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<DiscountResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateShippingMethodRequest {
    #[prost(string, tag = "1")]
    pub method_name: ::prost::alloc::string::String,
    #[prost(double, tag = "2")]
    pub cost: f64,
    #[prost(string, tag = "3")]
    pub estimated_delivery_time: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShippingMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateShippingMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
    #[prost(string, optional, tag = "2")]
    pub method_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag = "3")]
    pub cost: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "4")]
    pub estimated_delivery_time: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteShippingMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShippingMethodResponse {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
    #[prost(string, tag = "2")]
    pub method_name: ::prost::alloc::string::String,
    #[prost(double, tag = "3")]
    pub cost: f64,
    #[prost(string, tag = "4")]
    pub estimated_delivery_time: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShippingMethodsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ShippingMethodResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRoleRequest {
    #[prost(string, tag = "1")]
    pub role_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRoleRequest {
    #[prost(int64, tag = "1")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserRoleRequest {
    #[prost(int64, tag = "1")]
    pub role_id: i64,
    #[prost(string, tag = "2")]
    pub role_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserRoleRequest {
    #[prost(int64, tag = "1")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRoleResponse {
    #[prost(int64, tag = "1")]
    pub role_id: i64,
    #[prost(string, tag = "2")]
    pub role_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRolesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<UserRoleResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRolesMappingRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRolesMappingRequest {
    #[prost(int64, tag = "1")]
    pub id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserRolesMappingRequest {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, optional, tag = "2")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub role_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserRolesMappingRequest {
    #[prost(int64, tag = "1")]
    pub id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRolesMappingResponse {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRolesMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<UserRolesMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateTransactionRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(double, tag = "2")]
    pub amount: f64,
    #[prost(string, tag = "3")]
    pub r#type: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetTransactionRequest {
    #[prost(int64, tag = "1")]
    pub transaction_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateTransactionRequest {
    #[prost(int64, tag = "1")]
    pub transaction_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "3")]
    pub amount: ::core::option::Option<f64>,
    #[prost(string, optional, tag = "4")]
    pub r#type: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteTransactionRequest {
    #[prost(int64, tag = "1")]
    pub transaction_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionResponse {
    #[prost(int64, tag = "1")]
    pub transaction_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(double, tag = "3")]
    pub amount: f64,
    #[prost(string, tag = "4")]
    pub transaction_date: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub r#type: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<TransactionResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateNewsletterSubscriberRequest {
    #[prost(string, tag = "1")]
    pub email: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNewsletterSubscriberRequest {
    #[prost(int64, tag = "1")]
    pub subscriber_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateNewsletterSubscriberRequest {
    #[prost(int64, tag = "1")]
    pub subscriber_id: i64,
    #[prost(string, tag = "2")]
    pub email: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteNewsletterSubscriberRequest {
    #[prost(int64, tag = "1")]
    pub subscriber_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewsletterSubscriberResponse {
    #[prost(int64, tag = "1")]
    pub subscriber_id: i64,
    #[prost(string, tag = "2")]
    pub email: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub subscription_date: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NewsletterSubscribersResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<NewsletterSubscriberResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductRatingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(int64, tag = "3")]
    pub rating: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductRatingRequest {
    #[prost(int64, tag = "1")]
    pub rating_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductRatingRequest {
    #[prost(int64, tag = "1")]
    pub rating_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub rating: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductRatingRequest {
    #[prost(int64, tag = "1")]
    pub rating_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductRatingResponse {
    #[prost(int64, tag = "1")]
    pub rating_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub user_id: i64,
    #[prost(int64, tag = "4")]
    pub rating: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductRatingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductRatingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSizeRequest {
    #[prost(string, tag = "1")]
    pub size_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSizeRequest {
    #[prost(int64, tag = "1")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateSizeRequest {
    #[prost(int64, tag = "1")]
    pub size_id: i64,
    #[prost(string, tag = "2")]
    pub size_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteSizeRequest {
    #[prost(int64, tag = "1")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SizeResponse {
    #[prost(int64, tag = "1")]
    pub size_id: i64,
    #[prost(string, tag = "2")]
    pub size_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SizesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<SizeResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateColorRequest {
    #[prost(string, tag = "1")]
    pub color_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetColorRequest {
    #[prost(int64, tag = "1")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateColorRequest {
    #[prost(int64, tag = "1")]
    pub color_id: i64,
    #[prost(string, tag = "2")]
    pub color_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteColorRequest {
    #[prost(int64, tag = "1")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ColorResponse {
    #[prost(int64, tag = "1")]
    pub color_id: i64,
    #[prost(string, tag = "2")]
    pub color_name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ColorsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ColorResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductCategoryMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductCategoryMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductCategoryMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductCategoryMappingResponse {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub category_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductCategoryMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductCategoryMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductAttributeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductAttributeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductAttributeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductAttributeMappingResponse {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub attribute_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductAttributeMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductAttributeMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRoleMappingRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRoleMappingRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserRoleMappingRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRoleMappingResponse {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(int64, tag = "2")]
    pub role_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserRoleMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<UserRoleMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductSizeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductSizeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductSizeMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductSizeMappingResponse {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub size_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductSizeMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductSizeMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductColorMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductColorMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductColorMappingRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductColorMappingResponse {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub color_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductColorMappingsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductColorMappingResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateProductVariantRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub size_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub color_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "4")]
    pub additional_price: ::core::option::Option<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetProductVariantRequest {
    #[prost(int64, tag = "1")]
    pub variant_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateProductVariantRequest {
    #[prost(int64, tag = "1")]
    pub variant_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub size_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub color_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "5")]
    pub additional_price: ::core::option::Option<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteProductVariantRequest {
    #[prost(int64, tag = "1")]
    pub variant_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductVariantResponse {
    #[prost(int64, tag = "1")]
    pub variant_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, optional, tag = "3")]
    pub size_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "4")]
    pub color_id: ::core::option::Option<i64>,
    #[prost(double, optional, tag = "5")]
    pub additional_price: ::core::option::Option<f64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProductVariantsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ProductVariantResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateEventLogRequest {
    #[prost(string, tag = "1")]
    pub event_type: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub event_description: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub user_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetEventLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateEventLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(string, optional, tag = "2")]
    pub event_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub event_description: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int64, optional, tag = "4")]
    pub user_id: ::core::option::Option<i64>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteEventLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventLogResponse {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(string, tag = "2")]
    pub event_type: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub event_description: ::prost::alloc::string::String,
    #[prost(int64, tag = "4")]
    pub user_id: i64,
    #[prost(string, tag = "5")]
    pub event_time: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventLogsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<EventLogResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserActivityRequest {
    #[prost(int64, tag = "1")]
    pub user_id: i64,
    #[prost(string, tag = "2")]
    pub activity_type: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub activity_details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserActivityRequest {
    #[prost(int64, tag = "1")]
    pub activity_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserActivityRequest {
    #[prost(int64, tag = "1")]
    pub activity_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub user_id: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "3")]
    pub activity_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub activity_details: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteUserActivityRequest {
    #[prost(int64, tag = "1")]
    pub activity_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserActivityResponse {
    #[prost(int64, tag = "1")]
    pub activity_id: i64,
    #[prost(int64, tag = "2")]
    pub user_id: i64,
    #[prost(string, tag = "3")]
    pub activity_type: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub activity_time: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub activity_details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserActivitiesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<UserActivityResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateInventoryLogRequest {
    #[prost(int64, tag = "1")]
    pub product_id: i64,
    #[prost(int64, tag = "2")]
    pub change_quantity: i64,
    #[prost(string, tag = "3")]
    pub reason: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInventoryLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInventoryLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, optional, tag = "2")]
    pub product_id: ::core::option::Option<i64>,
    #[prost(int64, optional, tag = "3")]
    pub change_quantity: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "4")]
    pub reason: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteInventoryLogRequest {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InventoryLogResponse {
    #[prost(int64, tag = "1")]
    pub log_id: i64,
    #[prost(int64, tag = "2")]
    pub product_id: i64,
    #[prost(int64, tag = "3")]
    pub change_quantity: i64,
    #[prost(string, tag = "4")]
    pub log_time: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub reason: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InventoryLogsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<InventoryLogResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePromotionRequest {
    #[prost(string, tag = "1")]
    pub promotion_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub start_date: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub end_date: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPromotionRequest {
    #[prost(int64, tag = "1")]
    pub promotion_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdatePromotionRequest {
    #[prost(int64, tag = "1")]
    pub promotion_id: i64,
    #[prost(string, optional, tag = "2")]
    pub promotion_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub start_date: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "4")]
    pub end_date: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub details: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePromotionRequest {
    #[prost(int64, tag = "1")]
    pub promotion_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PromotionResponse {
    #[prost(int64, tag = "1")]
    pub promotion_id: i64,
    #[prost(string, tag = "2")]
    pub promotion_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub start_date: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub end_date: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PromotionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<PromotionResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateShippingZoneRequest {
    #[prost(string, tag = "1")]
    pub zone_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShippingZoneRequest {
    #[prost(int64, tag = "1")]
    pub zone_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateShippingZoneRequest {
    #[prost(int64, tag = "1")]
    pub zone_id: i64,
    #[prost(string, optional, tag = "2")]
    pub zone_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub description: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteShippingZoneRequest {
    #[prost(int64, tag = "1")]
    pub zone_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShippingZoneResponse {
    #[prost(int64, tag = "1")]
    pub zone_id: i64,
    #[prost(string, tag = "2")]
    pub zone_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShippingZonesResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<ShippingZoneResponse>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePaymentMethodRequest {
    #[prost(string, tag = "1")]
    pub method_name: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPaymentMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdatePaymentMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
    #[prost(string, optional, tag = "2")]
    pub method_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub details: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeletePaymentMethodRequest {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentMethodResponse {
    #[prost(int64, tag = "1")]
    pub method_id: i64,
    #[prost(string, tag = "2")]
    pub method_name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub details: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentMethodsResponse {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<PaymentMethodResponse>,
}
/// Generated client implementations.
pub mod grpc_services_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct GrpcServicesClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GrpcServicesClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GrpcServicesClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GrpcServicesClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            GrpcServicesClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Cart
        pub async fn create_cart_item(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateCartItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateCartItem"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_cart_items(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCartItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetCartItems",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetCartItems"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_cart_item(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateCartItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateCartItem"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_cart_item(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteCartItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteCartItem"));
            self.inner.unary(req, path, codec).await
        }
        /// Product
        pub async fn create_product(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProduct",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateProduct"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn search_product(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/SearchProduct",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "SearchProduct"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_product(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateProduct",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateProduct"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProduct",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteProduct"));
            self.inner.unary(req, path, codec).await
        }
        /// Users
        pub async fn create_user(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateUser"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UserResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetUser"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_user(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateUser"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteUser",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteUser"));
            self.inner.unary(req, path, codec).await
        }
        /// Categories
        pub async fn create_category(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateCategory"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_category(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoryResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetCategory"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_category(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateCategory"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_category(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteCategory"));
            self.inner.unary(req, path, codec).await
        }
        /// Orders
        pub async fn create_order(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateOrder",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateOrder"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_order(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetOrder",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetOrder"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_order(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateOrder",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateOrder"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_order(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteOrder",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteOrder"));
            self.inner.unary(req, path, codec).await
        }
        /// OrderDetails
        pub async fn create_order_detail(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateOrderDetail",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateOrderDetail"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_order_detail(
            &mut self,
            request: impl tonic::IntoRequest<super::GetOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetOrderDetail",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetOrderDetail"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_order_detail(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateOrderDetail",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateOrderDetail"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_order_detail(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteOrderDetail",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteOrderDetail"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Reviews
        pub async fn create_review(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateReviewRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReviewsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateReview",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateReview"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_review(
            &mut self,
            request: impl tonic::IntoRequest<super::GetReviewRequest>,
        ) -> std::result::Result<tonic::Response<super::ReviewResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetReview",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetReview"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_review(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateReviewRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReviewsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateReview",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateReview"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_review(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteReviewRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ReviewsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteReview",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteReview"));
            self.inner.unary(req, path, codec).await
        }
        /// ProductImages
        pub async fn create_product_image(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductImage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateProductImage"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_image(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImageResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductImage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetProductImage"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_product_image(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateProductImage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateProductImage"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_image(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductImage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteProductImage"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Suppliers
        pub async fn create_supplier(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateSupplier",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateSupplier"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_supplier(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SupplierResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetSupplier",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetSupplier"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_supplier(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateSupplier",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateSupplier"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_supplier(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteSupplier",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteSupplier"));
            self.inner.unary(req, path, codec).await
        }
        /// Inventory
        pub async fn create_inventory_item(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateInventoryItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateInventoryItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_inventory_item(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetInventoryItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetInventoryItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_inventory_item(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateInventoryItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateInventoryItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_inventory_item(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteInventoryItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteInventoryItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Wishlist
        pub async fn create_wishlist_item(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateWishlistItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateWishlistItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_wishlist_item(
            &mut self,
            request: impl tonic::IntoRequest<super::GetWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetWishlistItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetWishlistItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_wishlist_item(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateWishlistItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateWishlistItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_wishlist_item(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteWishlistItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteWishlistItem"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductAttributes
        pub async fn create_product_attribute(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductAttribute",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateProductAttribute",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_attribute(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductAttribute",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetProductAttribute"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_product_attribute(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateProductAttribute",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "UpdateProductAttribute",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_attribute(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductAttribute",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteProductAttribute",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Discounts
        pub async fn create_discount(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateDiscount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateDiscount"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_discount(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetDiscount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetDiscount"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_discount(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateDiscount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateDiscount"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_discount(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteDiscount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteDiscount"));
            self.inner.unary(req, path, codec).await
        }
        /// ShippingMethods
        pub async fn create_shipping_method(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateShippingMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateShippingMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_shipping_method(
            &mut self,
            request: impl tonic::IntoRequest<super::GetShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetShippingMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetShippingMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_shipping_method(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateShippingMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateShippingMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_shipping_method(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteShippingMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteShippingMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// UserRoles
        pub async fn create_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateUserRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateUserRole"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetUserRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetUserRole"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateUserRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateUserRole"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user_role(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteUserRole",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteUserRole"));
            self.inner.unary(req, path, codec).await
        }
        /// UserRolesMapping
        pub async fn create_user_roles_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateUserRolesMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateUserRolesMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_roles_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetUserRolesMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetUserRolesMapping"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_user_roles_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateUserRolesMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "UpdateUserRolesMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user_roles_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteUserRolesMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteUserRolesMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Transactions
        pub async fn create_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateTransaction",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateTransaction"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetTransaction",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetTransaction"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateTransaction",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateTransaction"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_transaction(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteTransaction",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteTransaction"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// NewsletterSubscribers
        pub async fn create_newsletter_subscriber(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateNewsletterSubscriber",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateNewsletterSubscriber",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_newsletter_subscriber(
            &mut self,
            request: impl tonic::IntoRequest<super::GetNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscriberResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetNewsletterSubscriber",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "GetNewsletterSubscriber",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_newsletter_subscriber(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateNewsletterSubscriber",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "UpdateNewsletterSubscriber",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_newsletter_subscriber(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteNewsletterSubscriber",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteNewsletterSubscriber",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductRatings
        pub async fn create_product_rating(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductRating",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateProductRating"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_rating(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductRating",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetProductRating"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_product_rating(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateProductRating",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateProductRating"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_rating(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductRating",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteProductRating"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Sizes
        pub async fn create_size(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateSize",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateSize"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_size(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizeResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetSize",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetSize"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_size(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateSize",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateSize"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_size(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteSize",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteSize"));
            self.inner.unary(req, path, codec).await
        }
        /// Colors
        pub async fn create_color(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateColor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateColor"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_color(
            &mut self,
            request: impl tonic::IntoRequest<super::GetColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetColor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetColor"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_color(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateColor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateColor"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_color(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteColor",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteColor"));
            self.inner.unary(req, path, codec).await
        }
        /// ProductCategoryMapping
        pub async fn create_product_category_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductCategoryMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateProductCategoryMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_category_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductCategoryMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "GetProductCategoryMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_category_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductCategoryMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteProductCategoryMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductAttributeMapping
        pub async fn create_product_attribute_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductAttributeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateProductAttributeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_attribute_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductAttributeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "GetProductAttributeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_attribute_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductAttributeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteProductAttributeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// UserRoleMapping
        pub async fn create_user_role_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateUserRoleMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateUserRoleMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_role_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetUserRoleMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetUserRoleMapping"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user_role_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteUserRoleMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteUserRoleMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductSizeMapping
        pub async fn create_product_size_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductSizeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateProductSizeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_size_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductSizeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "GetProductSizeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_size_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductSizeMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteProductSizeMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductColorMapping
        pub async fn create_product_color_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductColorMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "CreateProductColorMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_color_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductColorMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "GetProductColorMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_color_mapping(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductColorMapping",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "grpc_services.GRPCServices",
                        "DeleteProductColorMapping",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ProductVariants
        pub async fn create_product_variant(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateProductVariant",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateProductVariant"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_product_variant(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetProductVariant",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetProductVariant"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_product_variant(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateProductVariant",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateProductVariant"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_product_variant(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteProductVariant",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteProductVariant"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// EventLogs
        pub async fn create_event_log(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateEventLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "CreateEventLog"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_event_log(
            &mut self,
            request: impl tonic::IntoRequest<super::GetEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetEventLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetEventLog"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_event_log(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateEventLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "UpdateEventLog"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_event_log(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteEventLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "DeleteEventLog"));
            self.inner.unary(req, path, codec).await
        }
        /// UserActivity
        pub async fn create_user_activity(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateUserActivity",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateUserActivity"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_activity(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivityResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetUserActivity",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetUserActivity"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_user_activity(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateUserActivity",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateUserActivity"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user_activity(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteUserActivity",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteUserActivity"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// InventoryLog
        pub async fn create_inventory_log(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateInventoryLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateInventoryLog"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_inventory_log(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetInventoryLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetInventoryLog"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_inventory_log(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateInventoryLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateInventoryLog"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_inventory_log(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteInventoryLog",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteInventoryLog"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Promotions
        pub async fn create_promotion(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreatePromotion",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreatePromotion"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_promotion(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetPromotion",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("grpc_services.GRPCServices", "GetPromotion"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_promotion(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdatePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdatePromotion",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdatePromotion"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_promotion(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeletePromotion",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeletePromotion"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// ShippingZones
        pub async fn create_shipping_zone(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreateShippingZone",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreateShippingZone"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_shipping_zone(
            &mut self,
            request: impl tonic::IntoRequest<super::GetShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZoneResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetShippingZone",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetShippingZone"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_shipping_zone(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdateShippingZone",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdateShippingZone"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_shipping_zone(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeleteShippingZone",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeleteShippingZone"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// PaymentMethods
        pub async fn create_payment_method(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/CreatePaymentMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "CreatePaymentMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_payment_method(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/GetPaymentMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "GetPaymentMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_payment_method(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdatePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/UpdatePaymentMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "UpdatePaymentMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_payment_method(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/grpc_services.GRPCServices/DeletePaymentMethod",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("grpc_services.GRPCServices", "DeletePaymentMethod"),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod grpc_services_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with GrpcServicesServer.
    #[async_trait]
    pub trait GrpcServices: Send + Sync + 'static {
        /// Cart
        async fn create_cart_item(
            &self,
            request: tonic::Request<super::CreateCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        >;
        async fn get_cart_items(
            &self,
            request: tonic::Request<super::GetCartItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        >;
        async fn update_cart_item(
            &self,
            request: tonic::Request<super::UpdateCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        >;
        async fn delete_cart_item(
            &self,
            request: tonic::Request<super::DeleteCartItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CartItemsResponse>,
            tonic::Status,
        >;
        /// Product
        async fn create_product(
            &self,
            request: tonic::Request<super::CreateProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        >;
        async fn search_product(
            &self,
            request: tonic::Request<super::SearchProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        >;
        async fn update_product(
            &self,
            request: tonic::Request<super::UpdateProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        >;
        async fn delete_product(
            &self,
            request: tonic::Request<super::DeleteProductRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductsResponse>,
            tonic::Status,
        >;
        /// Users
        async fn create_user(
            &self,
            request: tonic::Request<super::CreateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status>;
        async fn get_user(
            &self,
            request: tonic::Request<super::GetUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UserResponse>, tonic::Status>;
        async fn update_user(
            &self,
            request: tonic::Request<super::UpdateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status>;
        async fn delete_user(
            &self,
            request: tonic::Request<super::DeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::UsersResponse>, tonic::Status>;
        /// Categories
        async fn create_category(
            &self,
            request: tonic::Request<super::CreateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        >;
        async fn get_category(
            &self,
            request: tonic::Request<super::GetCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoryResponse>,
            tonic::Status,
        >;
        async fn update_category(
            &self,
            request: tonic::Request<super::UpdateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        >;
        async fn delete_category(
            &self,
            request: tonic::Request<super::DeleteCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CategoriesResponse>,
            tonic::Status,
        >;
        /// Orders
        async fn create_order(
            &self,
            request: tonic::Request<super::CreateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status>;
        async fn get_order(
            &self,
            request: tonic::Request<super::GetOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrderResponse>, tonic::Status>;
        async fn update_order(
            &self,
            request: tonic::Request<super::UpdateOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status>;
        async fn delete_order(
            &self,
            request: tonic::Request<super::DeleteOrderRequest>,
        ) -> std::result::Result<tonic::Response<super::OrdersResponse>, tonic::Status>;
        /// OrderDetails
        async fn create_order_detail(
            &self,
            request: tonic::Request<super::CreateOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        >;
        async fn get_order_detail(
            &self,
            request: tonic::Request<super::GetOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailResponse>,
            tonic::Status,
        >;
        async fn update_order_detail(
            &self,
            request: tonic::Request<super::UpdateOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        >;
        async fn delete_order_detail(
            &self,
            request: tonic::Request<super::DeleteOrderDetailRequest>,
        ) -> std::result::Result<
            tonic::Response<super::OrderDetailsResponse>,
            tonic::Status,
        >;
        /// Reviews
        async fn create_review(
            &self,
            request: tonic::Request<super::CreateReviewRequest>,
        ) -> std::result::Result<tonic::Response<super::ReviewsResponse>, tonic::Status>;
        async fn get_review(
            &self,
            request: tonic::Request<super::GetReviewRequest>,
        ) -> std::result::Result<tonic::Response<super::ReviewResponse>, tonic::Status>;
        async fn update_review(
            &self,
            request: tonic::Request<super::UpdateReviewRequest>,
        ) -> std::result::Result<tonic::Response<super::ReviewsResponse>, tonic::Status>;
        async fn delete_review(
            &self,
            request: tonic::Request<super::DeleteReviewRequest>,
        ) -> std::result::Result<tonic::Response<super::ReviewsResponse>, tonic::Status>;
        /// ProductImages
        async fn create_product_image(
            &self,
            request: tonic::Request<super::CreateProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        >;
        async fn get_product_image(
            &self,
            request: tonic::Request<super::GetProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImageResponse>,
            tonic::Status,
        >;
        async fn update_product_image(
            &self,
            request: tonic::Request<super::UpdateProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        >;
        async fn delete_product_image(
            &self,
            request: tonic::Request<super::DeleteProductImageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductImagesResponse>,
            tonic::Status,
        >;
        /// Suppliers
        async fn create_supplier(
            &self,
            request: tonic::Request<super::CreateSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        >;
        async fn get_supplier(
            &self,
            request: tonic::Request<super::GetSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SupplierResponse>,
            tonic::Status,
        >;
        async fn update_supplier(
            &self,
            request: tonic::Request<super::UpdateSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        >;
        async fn delete_supplier(
            &self,
            request: tonic::Request<super::DeleteSupplierRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SuppliersResponse>,
            tonic::Status,
        >;
        /// Inventory
        async fn create_inventory_item(
            &self,
            request: tonic::Request<super::CreateInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        >;
        async fn get_inventory_item(
            &self,
            request: tonic::Request<super::GetInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemResponse>,
            tonic::Status,
        >;
        async fn update_inventory_item(
            &self,
            request: tonic::Request<super::UpdateInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        >;
        async fn delete_inventory_item(
            &self,
            request: tonic::Request<super::DeleteInventoryItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryItemsResponse>,
            tonic::Status,
        >;
        /// Wishlist
        async fn create_wishlist_item(
            &self,
            request: tonic::Request<super::CreateWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        >;
        async fn get_wishlist_item(
            &self,
            request: tonic::Request<super::GetWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemResponse>,
            tonic::Status,
        >;
        async fn update_wishlist_item(
            &self,
            request: tonic::Request<super::UpdateWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        >;
        async fn delete_wishlist_item(
            &self,
            request: tonic::Request<super::DeleteWishlistItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::WishlistItemsResponse>,
            tonic::Status,
        >;
        /// ProductAttributes
        async fn create_product_attribute(
            &self,
            request: tonic::Request<super::CreateProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        >;
        async fn get_product_attribute(
            &self,
            request: tonic::Request<super::GetProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeResponse>,
            tonic::Status,
        >;
        async fn update_product_attribute(
            &self,
            request: tonic::Request<super::UpdateProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        >;
        async fn delete_product_attribute(
            &self,
            request: tonic::Request<super::DeleteProductAttributeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributesResponse>,
            tonic::Status,
        >;
        /// Discounts
        async fn create_discount(
            &self,
            request: tonic::Request<super::CreateDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        >;
        async fn get_discount(
            &self,
            request: tonic::Request<super::GetDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountResponse>,
            tonic::Status,
        >;
        async fn update_discount(
            &self,
            request: tonic::Request<super::UpdateDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        >;
        async fn delete_discount(
            &self,
            request: tonic::Request<super::DeleteDiscountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DiscountsResponse>,
            tonic::Status,
        >;
        /// ShippingMethods
        async fn create_shipping_method(
            &self,
            request: tonic::Request<super::CreateShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        >;
        async fn get_shipping_method(
            &self,
            request: tonic::Request<super::GetShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodResponse>,
            tonic::Status,
        >;
        async fn update_shipping_method(
            &self,
            request: tonic::Request<super::UpdateShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        >;
        async fn delete_shipping_method(
            &self,
            request: tonic::Request<super::DeleteShippingMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingMethodsResponse>,
            tonic::Status,
        >;
        /// UserRoles
        async fn create_user_role(
            &self,
            request: tonic::Request<super::CreateUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        >;
        async fn get_user_role(
            &self,
            request: tonic::Request<super::GetUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleResponse>,
            tonic::Status,
        >;
        async fn update_user_role(
            &self,
            request: tonic::Request<super::UpdateUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        >;
        async fn delete_user_role(
            &self,
            request: tonic::Request<super::DeleteUserRoleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesResponse>,
            tonic::Status,
        >;
        /// UserRolesMapping
        async fn create_user_roles_mapping(
            &self,
            request: tonic::Request<super::CreateUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        >;
        async fn get_user_roles_mapping(
            &self,
            request: tonic::Request<super::GetUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingResponse>,
            tonic::Status,
        >;
        async fn update_user_roles_mapping(
            &self,
            request: tonic::Request<super::UpdateUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        >;
        async fn delete_user_roles_mapping(
            &self,
            request: tonic::Request<super::DeleteUserRolesMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRolesMappingsResponse>,
            tonic::Status,
        >;
        /// Transactions
        async fn create_transaction(
            &self,
            request: tonic::Request<super::CreateTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        >;
        async fn get_transaction(
            &self,
            request: tonic::Request<super::GetTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionResponse>,
            tonic::Status,
        >;
        async fn update_transaction(
            &self,
            request: tonic::Request<super::UpdateTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        >;
        async fn delete_transaction(
            &self,
            request: tonic::Request<super::DeleteTransactionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TransactionsResponse>,
            tonic::Status,
        >;
        /// NewsletterSubscribers
        async fn create_newsletter_subscriber(
            &self,
            request: tonic::Request<super::CreateNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        >;
        async fn get_newsletter_subscriber(
            &self,
            request: tonic::Request<super::GetNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscriberResponse>,
            tonic::Status,
        >;
        async fn update_newsletter_subscriber(
            &self,
            request: tonic::Request<super::UpdateNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        >;
        async fn delete_newsletter_subscriber(
            &self,
            request: tonic::Request<super::DeleteNewsletterSubscriberRequest>,
        ) -> std::result::Result<
            tonic::Response<super::NewsletterSubscribersResponse>,
            tonic::Status,
        >;
        /// ProductRatings
        async fn create_product_rating(
            &self,
            request: tonic::Request<super::CreateProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        >;
        async fn get_product_rating(
            &self,
            request: tonic::Request<super::GetProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingResponse>,
            tonic::Status,
        >;
        async fn update_product_rating(
            &self,
            request: tonic::Request<super::UpdateProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        >;
        async fn delete_product_rating(
            &self,
            request: tonic::Request<super::DeleteProductRatingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductRatingsResponse>,
            tonic::Status,
        >;
        /// Sizes
        async fn create_size(
            &self,
            request: tonic::Request<super::CreateSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status>;
        async fn get_size(
            &self,
            request: tonic::Request<super::GetSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizeResponse>, tonic::Status>;
        async fn update_size(
            &self,
            request: tonic::Request<super::UpdateSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status>;
        async fn delete_size(
            &self,
            request: tonic::Request<super::DeleteSizeRequest>,
        ) -> std::result::Result<tonic::Response<super::SizesResponse>, tonic::Status>;
        /// Colors
        async fn create_color(
            &self,
            request: tonic::Request<super::CreateColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status>;
        async fn get_color(
            &self,
            request: tonic::Request<super::GetColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorResponse>, tonic::Status>;
        async fn update_color(
            &self,
            request: tonic::Request<super::UpdateColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status>;
        async fn delete_color(
            &self,
            request: tonic::Request<super::DeleteColorRequest>,
        ) -> std::result::Result<tonic::Response<super::ColorsResponse>, tonic::Status>;
        /// ProductCategoryMapping
        async fn create_product_category_mapping(
            &self,
            request: tonic::Request<super::CreateProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingsResponse>,
            tonic::Status,
        >;
        async fn get_product_category_mapping(
            &self,
            request: tonic::Request<super::GetProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingResponse>,
            tonic::Status,
        >;
        async fn delete_product_category_mapping(
            &self,
            request: tonic::Request<super::DeleteProductCategoryMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductCategoryMappingsResponse>,
            tonic::Status,
        >;
        /// ProductAttributeMapping
        async fn create_product_attribute_mapping(
            &self,
            request: tonic::Request<super::CreateProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingsResponse>,
            tonic::Status,
        >;
        async fn get_product_attribute_mapping(
            &self,
            request: tonic::Request<super::GetProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingResponse>,
            tonic::Status,
        >;
        async fn delete_product_attribute_mapping(
            &self,
            request: tonic::Request<super::DeleteProductAttributeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductAttributeMappingsResponse>,
            tonic::Status,
        >;
        /// UserRoleMapping
        async fn create_user_role_mapping(
            &self,
            request: tonic::Request<super::CreateUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingsResponse>,
            tonic::Status,
        >;
        async fn get_user_role_mapping(
            &self,
            request: tonic::Request<super::GetUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingResponse>,
            tonic::Status,
        >;
        async fn delete_user_role_mapping(
            &self,
            request: tonic::Request<super::DeleteUserRoleMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserRoleMappingsResponse>,
            tonic::Status,
        >;
        /// ProductSizeMapping
        async fn create_product_size_mapping(
            &self,
            request: tonic::Request<super::CreateProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingsResponse>,
            tonic::Status,
        >;
        async fn get_product_size_mapping(
            &self,
            request: tonic::Request<super::GetProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingResponse>,
            tonic::Status,
        >;
        async fn delete_product_size_mapping(
            &self,
            request: tonic::Request<super::DeleteProductSizeMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductSizeMappingsResponse>,
            tonic::Status,
        >;
        /// ProductColorMapping
        async fn create_product_color_mapping(
            &self,
            request: tonic::Request<super::CreateProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingsResponse>,
            tonic::Status,
        >;
        async fn get_product_color_mapping(
            &self,
            request: tonic::Request<super::GetProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingResponse>,
            tonic::Status,
        >;
        async fn delete_product_color_mapping(
            &self,
            request: tonic::Request<super::DeleteProductColorMappingRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductColorMappingsResponse>,
            tonic::Status,
        >;
        /// ProductVariants
        async fn create_product_variant(
            &self,
            request: tonic::Request<super::CreateProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        >;
        async fn get_product_variant(
            &self,
            request: tonic::Request<super::GetProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantResponse>,
            tonic::Status,
        >;
        async fn update_product_variant(
            &self,
            request: tonic::Request<super::UpdateProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        >;
        async fn delete_product_variant(
            &self,
            request: tonic::Request<super::DeleteProductVariantRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ProductVariantsResponse>,
            tonic::Status,
        >;
        /// EventLogs
        async fn create_event_log(
            &self,
            request: tonic::Request<super::CreateEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        >;
        async fn get_event_log(
            &self,
            request: tonic::Request<super::GetEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogResponse>,
            tonic::Status,
        >;
        async fn update_event_log(
            &self,
            request: tonic::Request<super::UpdateEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        >;
        async fn delete_event_log(
            &self,
            request: tonic::Request<super::DeleteEventLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EventLogsResponse>,
            tonic::Status,
        >;
        /// UserActivity
        async fn create_user_activity(
            &self,
            request: tonic::Request<super::CreateUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        >;
        async fn get_user_activity(
            &self,
            request: tonic::Request<super::GetUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivityResponse>,
            tonic::Status,
        >;
        async fn update_user_activity(
            &self,
            request: tonic::Request<super::UpdateUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        >;
        async fn delete_user_activity(
            &self,
            request: tonic::Request<super::DeleteUserActivityRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UserActivitiesResponse>,
            tonic::Status,
        >;
        /// InventoryLog
        async fn create_inventory_log(
            &self,
            request: tonic::Request<super::CreateInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        >;
        async fn get_inventory_log(
            &self,
            request: tonic::Request<super::GetInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogResponse>,
            tonic::Status,
        >;
        async fn update_inventory_log(
            &self,
            request: tonic::Request<super::UpdateInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        >;
        async fn delete_inventory_log(
            &self,
            request: tonic::Request<super::DeleteInventoryLogRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InventoryLogsResponse>,
            tonic::Status,
        >;
        /// Promotions
        async fn create_promotion(
            &self,
            request: tonic::Request<super::CreatePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        >;
        async fn get_promotion(
            &self,
            request: tonic::Request<super::GetPromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionResponse>,
            tonic::Status,
        >;
        async fn update_promotion(
            &self,
            request: tonic::Request<super::UpdatePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        >;
        async fn delete_promotion(
            &self,
            request: tonic::Request<super::DeletePromotionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PromotionsResponse>,
            tonic::Status,
        >;
        /// ShippingZones
        async fn create_shipping_zone(
            &self,
            request: tonic::Request<super::CreateShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        >;
        async fn get_shipping_zone(
            &self,
            request: tonic::Request<super::GetShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZoneResponse>,
            tonic::Status,
        >;
        async fn update_shipping_zone(
            &self,
            request: tonic::Request<super::UpdateShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        >;
        async fn delete_shipping_zone(
            &self,
            request: tonic::Request<super::DeleteShippingZoneRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ShippingZonesResponse>,
            tonic::Status,
        >;
        /// PaymentMethods
        async fn create_payment_method(
            &self,
            request: tonic::Request<super::CreatePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        >;
        async fn get_payment_method(
            &self,
            request: tonic::Request<super::GetPaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodResponse>,
            tonic::Status,
        >;
        async fn update_payment_method(
            &self,
            request: tonic::Request<super::UpdatePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        >;
        async fn delete_payment_method(
            &self,
            request: tonic::Request<super::DeletePaymentMethodRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PaymentMethodsResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct GrpcServicesServer<T: GrpcServices> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: GrpcServices> GrpcServicesServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GrpcServicesServer<T>
    where
        T: GrpcServices,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/grpc_services.GRPCServices/CreateCartItem" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCartItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateCartItemRequest>
                    for CreateCartItemSvc<T> {
                        type Response = super::CartItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCartItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_cart_item(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateCartItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetCartItems" => {
                    #[allow(non_camel_case_types)]
                    struct GetCartItemsSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetCartItemsRequest>
                    for GetCartItemsSvc<T> {
                        type Response = super::CartItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCartItemsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_cart_items(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCartItemsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateCartItem" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateCartItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateCartItemRequest>
                    for UpdateCartItemSvc<T> {
                        type Response = super::CartItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateCartItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_cart_item(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateCartItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteCartItem" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCartItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteCartItemRequest>
                    for DeleteCartItemSvc<T> {
                        type Response = super::CartItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCartItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_cart_item(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteCartItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProduct" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductRequest>
                    for CreateProductSvc<T> {
                        type Response = super::ProductsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProductRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/SearchProduct" => {
                    #[allow(non_camel_case_types)]
                    struct SearchProductSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::SearchProductRequest>
                    for SearchProductSvc<T> {
                        type Response = super::ProductsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchProductRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::search_product(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SearchProductSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateProduct" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProductSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateProductRequest>
                    for UpdateProductSvc<T> {
                        type Response = super::ProductsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProductRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_product(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateProductSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProduct" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductRequest>
                    for DeleteProductSvc<T> {
                        type Response = super::ProductsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProductRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateUser" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateUserRequest>
                    for CreateUserSvc<T> {
                        type Response = super::UsersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetUser" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetUserRequest>
                    for GetUserSvc<T> {
                        type Response = super::UserResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateUser" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateUserSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateUserRequest>
                    for UpdateUserSvc<T> {
                        type Response = super::UsersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteUser" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteUserRequest>
                    for DeleteUserSvc<T> {
                        type Response = super::UsersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_user(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateCategory" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCategorySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateCategoryRequest>
                    for CreateCategorySvc<T> {
                        type Response = super::CategoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_category(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateCategorySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetCategory" => {
                    #[allow(non_camel_case_types)]
                    struct GetCategorySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetCategoryRequest>
                    for GetCategorySvc<T> {
                        type Response = super::CategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_category(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetCategorySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateCategory" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateCategorySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateCategoryRequest>
                    for UpdateCategorySvc<T> {
                        type Response = super::CategoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_category(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateCategorySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteCategory" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCategorySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteCategoryRequest>
                    for DeleteCategorySvc<T> {
                        type Response = super::CategoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_category(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteCategorySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateOrder" => {
                    #[allow(non_camel_case_types)]
                    struct CreateOrderSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateOrderRequest>
                    for CreateOrderSvc<T> {
                        type Response = super::OrdersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateOrderRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_order(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetOrder" => {
                    #[allow(non_camel_case_types)]
                    struct GetOrderSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetOrderRequest>
                    for GetOrderSvc<T> {
                        type Response = super::OrderResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetOrderRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_order(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateOrder" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateOrderSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateOrderRequest>
                    for UpdateOrderSvc<T> {
                        type Response = super::OrdersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateOrderRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_order(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteOrder" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteOrderSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteOrderRequest>
                    for DeleteOrderSvc<T> {
                        type Response = super::OrdersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteOrderRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_order(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteOrderSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateOrderDetail" => {
                    #[allow(non_camel_case_types)]
                    struct CreateOrderDetailSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateOrderDetailRequest>
                    for CreateOrderDetailSvc<T> {
                        type Response = super::OrderDetailsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateOrderDetailRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_order_detail(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateOrderDetailSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetOrderDetail" => {
                    #[allow(non_camel_case_types)]
                    struct GetOrderDetailSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetOrderDetailRequest>
                    for GetOrderDetailSvc<T> {
                        type Response = super::OrderDetailResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetOrderDetailRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_order_detail(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetOrderDetailSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateOrderDetail" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateOrderDetailSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateOrderDetailRequest>
                    for UpdateOrderDetailSvc<T> {
                        type Response = super::OrderDetailsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateOrderDetailRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_order_detail(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateOrderDetailSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteOrderDetail" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteOrderDetailSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteOrderDetailRequest>
                    for DeleteOrderDetailSvc<T> {
                        type Response = super::OrderDetailsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteOrderDetailRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_order_detail(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteOrderDetailSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateReview" => {
                    #[allow(non_camel_case_types)]
                    struct CreateReviewSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateReviewRequest>
                    for CreateReviewSvc<T> {
                        type Response = super::ReviewsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateReviewRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_review(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateReviewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetReview" => {
                    #[allow(non_camel_case_types)]
                    struct GetReviewSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetReviewRequest>
                    for GetReviewSvc<T> {
                        type Response = super::ReviewResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetReviewRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_review(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetReviewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateReview" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateReviewSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateReviewRequest>
                    for UpdateReviewSvc<T> {
                        type Response = super::ReviewsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateReviewRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_review(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateReviewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteReview" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteReviewSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteReviewRequest>
                    for DeleteReviewSvc<T> {
                        type Response = super::ReviewsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteReviewRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_review(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteReviewSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductImage" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductImageSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductImageRequest>
                    for CreateProductImageSvc<T> {
                        type Response = super::ProductImagesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProductImageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_image(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductImageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductImage" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductImageSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductImageRequest>
                    for GetProductImageSvc<T> {
                        type Response = super::ProductImageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductImageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_image(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductImageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateProductImage" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProductImageSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateProductImageRequest>
                    for UpdateProductImageSvc<T> {
                        type Response = super::ProductImagesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProductImageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_product_image(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateProductImageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductImage" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductImageSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductImageRequest>
                    for DeleteProductImageSvc<T> {
                        type Response = super::ProductImagesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProductImageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_image(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductImageSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateSupplier" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSupplierSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateSupplierRequest>
                    for CreateSupplierSvc<T> {
                        type Response = super::SuppliersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSupplierRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_supplier(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateSupplierSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetSupplier" => {
                    #[allow(non_camel_case_types)]
                    struct GetSupplierSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetSupplierRequest>
                    for GetSupplierSvc<T> {
                        type Response = super::SupplierResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetSupplierRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_supplier(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetSupplierSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateSupplier" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSupplierSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateSupplierRequest>
                    for UpdateSupplierSvc<T> {
                        type Response = super::SuppliersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateSupplierRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_supplier(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateSupplierSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteSupplier" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSupplierSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteSupplierRequest>
                    for DeleteSupplierSvc<T> {
                        type Response = super::SuppliersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteSupplierRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_supplier(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteSupplierSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateInventoryItem" => {
                    #[allow(non_camel_case_types)]
                    struct CreateInventoryItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateInventoryItemRequest>
                    for CreateInventoryItemSvc<T> {
                        type Response = super::InventoryItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateInventoryItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_inventory_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateInventoryItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetInventoryItem" => {
                    #[allow(non_camel_case_types)]
                    struct GetInventoryItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetInventoryItemRequest>
                    for GetInventoryItemSvc<T> {
                        type Response = super::InventoryItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetInventoryItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_inventory_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetInventoryItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateInventoryItem" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateInventoryItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateInventoryItemRequest>
                    for UpdateInventoryItemSvc<T> {
                        type Response = super::InventoryItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateInventoryItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_inventory_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateInventoryItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteInventoryItem" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteInventoryItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteInventoryItemRequest>
                    for DeleteInventoryItemSvc<T> {
                        type Response = super::InventoryItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteInventoryItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_inventory_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteInventoryItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateWishlistItem" => {
                    #[allow(non_camel_case_types)]
                    struct CreateWishlistItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateWishlistItemRequest>
                    for CreateWishlistItemSvc<T> {
                        type Response = super::WishlistItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateWishlistItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_wishlist_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateWishlistItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetWishlistItem" => {
                    #[allow(non_camel_case_types)]
                    struct GetWishlistItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetWishlistItemRequest>
                    for GetWishlistItemSvc<T> {
                        type Response = super::WishlistItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetWishlistItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_wishlist_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetWishlistItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateWishlistItem" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateWishlistItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateWishlistItemRequest>
                    for UpdateWishlistItemSvc<T> {
                        type Response = super::WishlistItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateWishlistItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_wishlist_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateWishlistItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteWishlistItem" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteWishlistItemSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteWishlistItemRequest>
                    for DeleteWishlistItemSvc<T> {
                        type Response = super::WishlistItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteWishlistItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_wishlist_item(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteWishlistItemSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductAttributeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductAttributeRequest>
                    for CreateProductAttributeSvc<T> {
                        type Response = super::ProductAttributesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProductAttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_attribute(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductAttributeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductAttributeRequest>
                    for GetProductAttributeSvc<T> {
                        type Response = super::ProductAttributeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductAttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_attribute(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateProductAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProductAttributeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateProductAttributeRequest>
                    for UpdateProductAttributeSvc<T> {
                        type Response = super::ProductAttributesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProductAttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_product_attribute(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateProductAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductAttribute" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductAttributeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductAttributeRequest>
                    for DeleteProductAttributeSvc<T> {
                        type Response = super::ProductAttributesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProductAttributeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_attribute(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductAttributeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateDiscount" => {
                    #[allow(non_camel_case_types)]
                    struct CreateDiscountSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateDiscountRequest>
                    for CreateDiscountSvc<T> {
                        type Response = super::DiscountsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateDiscountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_discount(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateDiscountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetDiscount" => {
                    #[allow(non_camel_case_types)]
                    struct GetDiscountSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetDiscountRequest>
                    for GetDiscountSvc<T> {
                        type Response = super::DiscountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetDiscountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_discount(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetDiscountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateDiscount" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateDiscountSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateDiscountRequest>
                    for UpdateDiscountSvc<T> {
                        type Response = super::DiscountsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateDiscountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_discount(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateDiscountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteDiscount" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteDiscountSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteDiscountRequest>
                    for DeleteDiscountSvc<T> {
                        type Response = super::DiscountsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteDiscountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_discount(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteDiscountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateShippingMethod" => {
                    #[allow(non_camel_case_types)]
                    struct CreateShippingMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateShippingMethodRequest>
                    for CreateShippingMethodSvc<T> {
                        type Response = super::ShippingMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateShippingMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_shipping_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateShippingMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetShippingMethod" => {
                    #[allow(non_camel_case_types)]
                    struct GetShippingMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetShippingMethodRequest>
                    for GetShippingMethodSvc<T> {
                        type Response = super::ShippingMethodResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetShippingMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_shipping_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetShippingMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateShippingMethod" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateShippingMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateShippingMethodRequest>
                    for UpdateShippingMethodSvc<T> {
                        type Response = super::ShippingMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateShippingMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_shipping_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateShippingMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteShippingMethod" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteShippingMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteShippingMethodRequest>
                    for DeleteShippingMethodSvc<T> {
                        type Response = super::ShippingMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteShippingMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_shipping_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteShippingMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateUserRole" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserRoleSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateUserRoleRequest>
                    for CreateUserRoleSvc<T> {
                        type Response = super::UserRolesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_user_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetUserRole" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserRoleSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetUserRoleRequest>
                    for GetUserRoleSvc<T> {
                        type Response = super::UserRoleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_user_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateUserRole" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateUserRoleSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateUserRoleRequest>
                    for UpdateUserRoleSvc<T> {
                        type Response = super::UserRolesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateUserRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_user_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateUserRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteUserRole" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserRoleSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteUserRoleRequest>
                    for DeleteUserRoleSvc<T> {
                        type Response = super::UserRolesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRoleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_user_role(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserRoleSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateUserRolesMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserRolesMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateUserRolesMappingRequest>
                    for CreateUserRolesMappingSvc<T> {
                        type Response = super::UserRolesMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRolesMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_user_roles_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserRolesMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetUserRolesMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserRolesMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetUserRolesMappingRequest>
                    for GetUserRolesMappingSvc<T> {
                        type Response = super::UserRolesMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserRolesMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_user_roles_mapping(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserRolesMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateUserRolesMapping" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateUserRolesMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateUserRolesMappingRequest>
                    for UpdateUserRolesMappingSvc<T> {
                        type Response = super::UserRolesMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateUserRolesMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_user_roles_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateUserRolesMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteUserRolesMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserRolesMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteUserRolesMappingRequest>
                    for DeleteUserRolesMappingSvc<T> {
                        type Response = super::UserRolesMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRolesMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_user_roles_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserRolesMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTransactionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateTransactionRequest>
                    for CreateTransactionSvc<T> {
                        type Response = super::TransactionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTransactionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_transaction(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct GetTransactionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetTransactionRequest>
                    for GetTransactionSvc<T> {
                        type Response = super::TransactionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTransactionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_transaction(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateTransactionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateTransactionRequest>
                    for UpdateTransactionSvc<T> {
                        type Response = super::TransactionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateTransactionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_transaction(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteTransaction" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTransactionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteTransactionRequest>
                    for DeleteTransactionSvc<T> {
                        type Response = super::TransactionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTransactionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_transaction(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteTransactionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateNewsletterSubscriber" => {
                    #[allow(non_camel_case_types)]
                    struct CreateNewsletterSubscriberSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::CreateNewsletterSubscriberRequest,
                    > for CreateNewsletterSubscriberSvc<T> {
                        type Response = super::NewsletterSubscribersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateNewsletterSubscriberRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_newsletter_subscriber(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateNewsletterSubscriberSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetNewsletterSubscriber" => {
                    #[allow(non_camel_case_types)]
                    struct GetNewsletterSubscriberSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetNewsletterSubscriberRequest>
                    for GetNewsletterSubscriberSvc<T> {
                        type Response = super::NewsletterSubscriberResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetNewsletterSubscriberRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_newsletter_subscriber(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetNewsletterSubscriberSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateNewsletterSubscriber" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateNewsletterSubscriberSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::UpdateNewsletterSubscriberRequest,
                    > for UpdateNewsletterSubscriberSvc<T> {
                        type Response = super::NewsletterSubscribersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::UpdateNewsletterSubscriberRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_newsletter_subscriber(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateNewsletterSubscriberSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteNewsletterSubscriber" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteNewsletterSubscriberSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::DeleteNewsletterSubscriberRequest,
                    > for DeleteNewsletterSubscriberSvc<T> {
                        type Response = super::NewsletterSubscribersResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteNewsletterSubscriberRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_newsletter_subscriber(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteNewsletterSubscriberSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductRating" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductRatingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductRatingRequest>
                    for CreateProductRatingSvc<T> {
                        type Response = super::ProductRatingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProductRatingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_rating(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductRatingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductRating" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductRatingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductRatingRequest>
                    for GetProductRatingSvc<T> {
                        type Response = super::ProductRatingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductRatingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_rating(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductRatingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateProductRating" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProductRatingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateProductRatingRequest>
                    for UpdateProductRatingSvc<T> {
                        type Response = super::ProductRatingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProductRatingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_product_rating(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateProductRatingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductRating" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductRatingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductRatingRequest>
                    for DeleteProductRatingSvc<T> {
                        type Response = super::ProductRatingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProductRatingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_rating(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductRatingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateSize" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSizeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateSizeRequest>
                    for CreateSizeSvc<T> {
                        type Response = super::SizesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSizeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_size(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateSizeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetSize" => {
                    #[allow(non_camel_case_types)]
                    struct GetSizeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetSizeRequest>
                    for GetSizeSvc<T> {
                        type Response = super::SizeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetSizeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_size(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetSizeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateSize" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateSizeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateSizeRequest>
                    for UpdateSizeSvc<T> {
                        type Response = super::SizesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateSizeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_size(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateSizeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteSize" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteSizeSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteSizeRequest>
                    for DeleteSizeSvc<T> {
                        type Response = super::SizesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteSizeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_size(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteSizeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateColor" => {
                    #[allow(non_camel_case_types)]
                    struct CreateColorSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateColorRequest>
                    for CreateColorSvc<T> {
                        type Response = super::ColorsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateColorRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_color(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateColorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetColor" => {
                    #[allow(non_camel_case_types)]
                    struct GetColorSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetColorRequest>
                    for GetColorSvc<T> {
                        type Response = super::ColorResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetColorRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_color(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetColorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateColor" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateColorSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateColorRequest>
                    for UpdateColorSvc<T> {
                        type Response = super::ColorsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateColorRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_color(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateColorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteColor" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteColorSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteColorRequest>
                    for DeleteColorSvc<T> {
                        type Response = super::ColorsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteColorRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_color(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteColorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductCategoryMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductCategoryMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::CreateProductCategoryMappingRequest,
                    > for CreateProductCategoryMappingSvc<T> {
                        type Response = super::ProductCategoryMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateProductCategoryMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_category_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductCategoryMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductCategoryMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductCategoryMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::GetProductCategoryMappingRequest,
                    > for GetProductCategoryMappingSvc<T> {
                        type Response = super::ProductCategoryMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetProductCategoryMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_category_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductCategoryMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductCategoryMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductCategoryMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::DeleteProductCategoryMappingRequest,
                    > for DeleteProductCategoryMappingSvc<T> {
                        type Response = super::ProductCategoryMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteProductCategoryMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_category_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductCategoryMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductAttributeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductAttributeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::CreateProductAttributeMappingRequest,
                    > for CreateProductAttributeMappingSvc<T> {
                        type Response = super::ProductAttributeMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateProductAttributeMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_attribute_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductAttributeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductAttributeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductAttributeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::GetProductAttributeMappingRequest,
                    > for GetProductAttributeMappingSvc<T> {
                        type Response = super::ProductAttributeMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetProductAttributeMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_attribute_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductAttributeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductAttributeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductAttributeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::DeleteProductAttributeMappingRequest,
                    > for DeleteProductAttributeMappingSvc<T> {
                        type Response = super::ProductAttributeMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteProductAttributeMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_attribute_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductAttributeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateUserRoleMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserRoleMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateUserRoleMappingRequest>
                    for CreateUserRoleMappingSvc<T> {
                        type Response = super::UserRoleMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRoleMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_user_role_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserRoleMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetUserRoleMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserRoleMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetUserRoleMappingRequest>
                    for GetUserRoleMappingSvc<T> {
                        type Response = super::UserRoleMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserRoleMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_user_role_mapping(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserRoleMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteUserRoleMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserRoleMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteUserRoleMappingRequest>
                    for DeleteUserRoleMappingSvc<T> {
                        type Response = super::UserRoleMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRoleMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_user_role_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserRoleMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductSizeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductSizeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductSizeMappingRequest>
                    for CreateProductSizeMappingSvc<T> {
                        type Response = super::ProductSizeMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateProductSizeMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_size_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductSizeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductSizeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductSizeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductSizeMappingRequest>
                    for GetProductSizeMappingSvc<T> {
                        type Response = super::ProductSizeMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductSizeMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_size_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductSizeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductSizeMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductSizeMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductSizeMappingRequest>
                    for DeleteProductSizeMappingSvc<T> {
                        type Response = super::ProductSizeMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteProductSizeMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_size_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductSizeMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductColorMapping" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductColorMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::CreateProductColorMappingRequest,
                    > for CreateProductColorMappingSvc<T> {
                        type Response = super::ProductColorMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateProductColorMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_color_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductColorMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductColorMapping" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductColorMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductColorMappingRequest>
                    for GetProductColorMappingSvc<T> {
                        type Response = super::ProductColorMappingResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductColorMappingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_color_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductColorMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductColorMapping" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductColorMappingSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<
                        super::DeleteProductColorMappingRequest,
                    > for DeleteProductColorMappingSvc<T> {
                        type Response = super::ProductColorMappingsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteProductColorMappingRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_color_mapping(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductColorMappingSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateProductVariant" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProductVariantSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateProductVariantRequest>
                    for CreateProductVariantSvc<T> {
                        type Response = super::ProductVariantsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProductVariantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_product_variant(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateProductVariantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetProductVariant" => {
                    #[allow(non_camel_case_types)]
                    struct GetProductVariantSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetProductVariantRequest>
                    for GetProductVariantSvc<T> {
                        type Response = super::ProductVariantResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProductVariantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_product_variant(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetProductVariantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateProductVariant" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProductVariantSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateProductVariantRequest>
                    for UpdateProductVariantSvc<T> {
                        type Response = super::ProductVariantsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProductVariantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_product_variant(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateProductVariantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteProductVariant" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProductVariantSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteProductVariantRequest>
                    for DeleteProductVariantSvc<T> {
                        type Response = super::ProductVariantsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProductVariantRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_product_variant(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteProductVariantSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateEventLog" => {
                    #[allow(non_camel_case_types)]
                    struct CreateEventLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateEventLogRequest>
                    for CreateEventLogSvc<T> {
                        type Response = super::EventLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateEventLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_event_log(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateEventLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetEventLog" => {
                    #[allow(non_camel_case_types)]
                    struct GetEventLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetEventLogRequest>
                    for GetEventLogSvc<T> {
                        type Response = super::EventLogResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetEventLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_event_log(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetEventLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateEventLog" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateEventLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateEventLogRequest>
                    for UpdateEventLogSvc<T> {
                        type Response = super::EventLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateEventLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_event_log(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateEventLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteEventLog" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteEventLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteEventLogRequest>
                    for DeleteEventLogSvc<T> {
                        type Response = super::EventLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteEventLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_event_log(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteEventLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateUserActivity" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserActivitySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateUserActivityRequest>
                    for CreateUserActivitySvc<T> {
                        type Response = super::UserActivitiesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserActivityRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_user_activity(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateUserActivitySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetUserActivity" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserActivitySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetUserActivityRequest>
                    for GetUserActivitySvc<T> {
                        type Response = super::UserActivityResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserActivityRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_user_activity(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetUserActivitySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateUserActivity" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateUserActivitySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateUserActivityRequest>
                    for UpdateUserActivitySvc<T> {
                        type Response = super::UserActivitiesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateUserActivityRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_user_activity(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateUserActivitySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteUserActivity" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserActivitySvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteUserActivityRequest>
                    for DeleteUserActivitySvc<T> {
                        type Response = super::UserActivitiesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserActivityRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_user_activity(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteUserActivitySvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateInventoryLog" => {
                    #[allow(non_camel_case_types)]
                    struct CreateInventoryLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateInventoryLogRequest>
                    for CreateInventoryLogSvc<T> {
                        type Response = super::InventoryLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateInventoryLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_inventory_log(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateInventoryLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetInventoryLog" => {
                    #[allow(non_camel_case_types)]
                    struct GetInventoryLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetInventoryLogRequest>
                    for GetInventoryLogSvc<T> {
                        type Response = super::InventoryLogResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetInventoryLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_inventory_log(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetInventoryLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateInventoryLog" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateInventoryLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateInventoryLogRequest>
                    for UpdateInventoryLogSvc<T> {
                        type Response = super::InventoryLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateInventoryLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_inventory_log(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateInventoryLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteInventoryLog" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteInventoryLogSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteInventoryLogRequest>
                    for DeleteInventoryLogSvc<T> {
                        type Response = super::InventoryLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteInventoryLogRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_inventory_log(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteInventoryLogSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreatePromotion" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePromotionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreatePromotionRequest>
                    for CreatePromotionSvc<T> {
                        type Response = super::PromotionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePromotionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_promotion(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreatePromotionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetPromotion" => {
                    #[allow(non_camel_case_types)]
                    struct GetPromotionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetPromotionRequest>
                    for GetPromotionSvc<T> {
                        type Response = super::PromotionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPromotionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_promotion(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPromotionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdatePromotion" => {
                    #[allow(non_camel_case_types)]
                    struct UpdatePromotionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdatePromotionRequest>
                    for UpdatePromotionSvc<T> {
                        type Response = super::PromotionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdatePromotionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_promotion(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdatePromotionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeletePromotion" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePromotionSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeletePromotionRequest>
                    for DeletePromotionSvc<T> {
                        type Response = super::PromotionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePromotionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_promotion(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeletePromotionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreateShippingZone" => {
                    #[allow(non_camel_case_types)]
                    struct CreateShippingZoneSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreateShippingZoneRequest>
                    for CreateShippingZoneSvc<T> {
                        type Response = super::ShippingZonesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateShippingZoneRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_shipping_zone(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateShippingZoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetShippingZone" => {
                    #[allow(non_camel_case_types)]
                    struct GetShippingZoneSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetShippingZoneRequest>
                    for GetShippingZoneSvc<T> {
                        type Response = super::ShippingZoneResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetShippingZoneRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_shipping_zone(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetShippingZoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdateShippingZone" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateShippingZoneSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdateShippingZoneRequest>
                    for UpdateShippingZoneSvc<T> {
                        type Response = super::ShippingZonesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateShippingZoneRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_shipping_zone(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdateShippingZoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeleteShippingZone" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteShippingZoneSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeleteShippingZoneRequest>
                    for DeleteShippingZoneSvc<T> {
                        type Response = super::ShippingZonesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteShippingZoneRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_shipping_zone(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeleteShippingZoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/CreatePaymentMethod" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePaymentMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::CreatePaymentMethodRequest>
                    for CreatePaymentMethodSvc<T> {
                        type Response = super::PaymentMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePaymentMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::create_payment_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreatePaymentMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/GetPaymentMethod" => {
                    #[allow(non_camel_case_types)]
                    struct GetPaymentMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::GetPaymentMethodRequest>
                    for GetPaymentMethodSvc<T> {
                        type Response = super::PaymentMethodResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPaymentMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::get_payment_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetPaymentMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/UpdatePaymentMethod" => {
                    #[allow(non_camel_case_types)]
                    struct UpdatePaymentMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::UpdatePaymentMethodRequest>
                    for UpdatePaymentMethodSvc<T> {
                        type Response = super::PaymentMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdatePaymentMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::update_payment_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = UpdatePaymentMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/grpc_services.GRPCServices/DeletePaymentMethod" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePaymentMethodSvc<T: GrpcServices>(pub Arc<T>);
                    impl<
                        T: GrpcServices,
                    > tonic::server::UnaryService<super::DeletePaymentMethodRequest>
                    for DeletePaymentMethodSvc<T> {
                        type Response = super::PaymentMethodsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePaymentMethodRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as GrpcServices>::delete_payment_method(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeletePaymentMethodSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: GrpcServices> Clone for GrpcServicesServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: GrpcServices> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: GrpcServices> tonic::server::NamedService for GrpcServicesServer<T> {
        const NAME: &'static str = "grpc_services.GRPCServices";
    }
}
