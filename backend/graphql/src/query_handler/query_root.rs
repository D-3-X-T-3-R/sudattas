use juniper::IntoFieldError;
use super::Context;
use crate::resolvers::{
    cart::{self, schema::Cart},
    coupons::{self, schema::{Coupon, ValidateCoupon}},
    discounts::{self, schema::{Discount, SearchDiscount}},
    order_events::{self, schema::OrderEvent},
    inventory::{self, schema::{InventoryItem, SearchInventoryItem}},
    reviews::{self, schema::{Review, SearchReview}},
    shipping_addresses::{self, schema::ShippingAddress},
    shipping_methods::{self, schema::{SearchShippingMethod, ShippingMethod}},
    shipping_zones::{self, schema::{SearchShippingZone, ShippingZone}},
    payment_intents::{self, schema::{GetPaymentIntent, PaymentIntent}},
    shipments::{self, schema::{GetShipment, Shipment}},
    category::{
        self,
        schema::{Category, SearchCategory},
    },
    country::{
        self,
        schema::{Country, SearchCountry},
    },
    orders::{
        self,
        schema::{Order, SearchOrder},
    },
    product::{
        self,
        schema::{Product, SearchProduct},
    },
    product_images::{
        self,
        schema::{GetPresignedUploadUrl, PresignedUploadUrl, ProductImage, SearchProductImage},
    },
    state::{
        self,
        schema::{SearchState, State},
    },
    wishlist::{
        self,
        schema::{SearchWishlistItem, WishlistItem},
    },
};
use juniper::FieldResult;

pub struct QueryRoot;

/// Minimal auth capability info; uses Context fields so they are not reported as dead code.
#[derive(juniper::GraphQLObject)]
struct AuthInfo {
    /// Whether session-based (guest) auth is enabled (REDIS_URL configured).
    session_enabled: bool,
    /// Number of JWKS keys loaded for JWT validation.
    jwks_key_count: i32,
    /// Current request’s user ID (JWT or session), if any.
    current_user_id: Option<String>,
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
        }
    }

    // Cart
    #[instrument(err, ret)]
    async fn get_cart_items(
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> FieldResult<Vec<Cart>> {
        cart::handlers::get_cart_items(user_id, session_id)
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
    async fn search_order(search: SearchOrder) -> FieldResult<Vec<Order>> {
        orders::handlers::search_order(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn search_wishlist_item(search: SearchWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::search_wishlist_item(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Country
    #[instrument(err, ret)]
    async fn search_country(search: SearchCountry) -> FieldResult<Vec<Country>> {
        country::handlers::search_country(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // State
    #[instrument(err, ret)]
    async fn search_state(search: SearchState) -> FieldResult<Vec<State>> {
        state::handlers::search_state(search)
            .await
            .map_err(|e| e.into_field_error())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn get_payment_intent(input: GetPaymentIntent) -> FieldResult<Vec<PaymentIntent>> {
        payment_intents::handlers::get_payment_intent(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipments
    #[instrument(err, ret)]
    async fn get_shipment(input: GetShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::get_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Coupons
    #[instrument(err, ret)]
    async fn validate_coupon(input: ValidateCoupon) -> FieldResult<Vec<Coupon>> {
        coupons::handlers::validate_coupon(input)
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

    // Inventory
    #[instrument(err, ret)]
    async fn search_inventory_item(input: SearchInventoryItem) -> FieldResult<Vec<InventoryItem>> {
        inventory::handlers::search_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product Images — R2 presigned upload
    #[instrument(err, ret)]
    async fn get_presigned_upload_url(
        input: GetPresignedUploadUrl,
    ) -> FieldResult<Vec<PresignedUploadUrl>> {
        product_images::handlers::get_presigned_upload_url(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order Events
    #[instrument(err, ret)]
    async fn get_order_events(order_id: String) -> FieldResult<Vec<OrderEvent>> {
        order_events::handlers::get_order_events(order_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Discounts
    #[instrument(err, ret)]
    async fn search_discount(input: SearchDiscount) -> FieldResult<Vec<Discount>> {
        discounts::handlers::search_discount(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping methods
    #[instrument(err, ret)]
    async fn search_shipping_method(input: SearchShippingMethod) -> FieldResult<Vec<ShippingMethod>> {
        shipping_methods::handlers::search_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping zones
    #[instrument(err, ret)]
    async fn search_shipping_zone(input: SearchShippingZone) -> FieldResult<Vec<ShippingZone>> {
        shipping_zones::handlers::search_shipping_zone(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping addresses
    #[instrument(err, ret)]
    async fn get_shipping_addresses() -> FieldResult<Vec<ShippingAddress>> {
        shipping_addresses::handlers::get_shipping_addresses()
            .await
            .map_err(|e| e.into_field_error())
    }
}
