use super::Context;
use crate::resolvers::{
    cart::{
        self,
        schema::{Cart, CartMutation, DeleteCartItem, NewCart},
    },
    category::{
        self,
        schema::{Category, CategoryMutation, NewCategory},
    },
    country::{
        self,
        schema::{Country, NewCountry},
    },
    coupons::{
        self,
        schema::{ApplyCoupon, Coupon},
    },
    discounts::{
        self,
        schema::{Discount, DiscountMutation, NewDiscount},
    },
    inventory::{
        self,
        schema::{InventoryItem, InventoryItemMutation, NewInventoryItem},
    },
    order_details::{
        self,
        schema::{NewOrderDetails, OrderDetails, OrderDetailsMutation},
    },
    order_events::{
        self,
        schema::{NewOrderEvent, OrderEvent},
    },
    orders::{
        self,
        schema::{NewOrder, Order, OrderMutation},
    },
    payment_intents::{
        self,
        schema::{CapturePayment, NewPaymentIntent, PaymentIntent},
    },
    product::{
        self,
        schema::{NewProduct, Product, ProductMutation},
    },
    product_images::{
        self,
        schema::{ConfirmImageUpload, ProductImage, ProductImageMutation},
    },
    reviews::{
        self,
        schema::{NewReview, Review, ReviewMutation},
    },
    shipments::{
        self,
        schema::{NewShipment, Shipment, UpdateShipment},
    },
    shipping_addresses::{
        self,
        schema::{NewShippingAddress, ShippingAddress, ShippingAddressMutation},
    },
    shipping_methods::{
        self,
        schema::{NewShippingMethod, ShippingMethod, ShippingMethodMutation},
    },
    shipping_zones::{
        self,
        schema::{NewShippingZone, ShippingZone, ShippingZoneMutation},
    },
    state::{
        self,
        schema::{NewState, State},
    },
    wishlist::{
        self,
        schema::{DeleteWishlistItem, NewWishlistItem, WishlistItem},
    },
};
use juniper::FieldResult;
use juniper::IntoFieldError;

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    // Cart
    #[instrument(err, ret)]
    async fn add_cart_item(cart_item: NewCart) -> FieldResult<Vec<Cart>> {
        cart::handlers::add_cart_item(cart_item)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_cart_item(delete: DeleteCartItem) -> FieldResult<Vec<Cart>> {
        cart::handlers::delete_cart_item(delete)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_cart_item(cart_item: CartMutation) -> FieldResult<Vec<Cart>> {
        cart::handlers::update_cart_item(cart_item)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product
    #[instrument(err, ret)]
    async fn create_product(product: NewProduct) -> FieldResult<Vec<Product>> {
        product::handlers::create_product(product)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product(product_id: String) -> FieldResult<Vec<Product>> {
        product::handlers::delete_product(product_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product(product: ProductMutation) -> FieldResult<Vec<Product>> {
        product::handlers::update_product(product)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Category
    #[instrument(err, ret)]
    async fn create_category(category: NewCategory) -> FieldResult<Vec<Category>> {
        category::handlers::create_category(category)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_category(category_id: String) -> FieldResult<Vec<Category>> {
        category::handlers::delete_category(category_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_category(category: CategoryMutation) -> FieldResult<Vec<Category>> {
        category::handlers::update_category(category)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order
    #[instrument(err, ret)]
    async fn place_order(context: &Context, order: NewOrder) -> FieldResult<Vec<Order>> {
        // Checkout requires a full login (JWT). Guest sessions (X-Session-Id only) are not
        // allowed to place orders — the client must authenticate first.
        let user_id = context
            .jwt_user_id()
            .ok_or_else(|| {
                juniper::FieldError::new("Login required to place an order", juniper::Value::null())
            })?
            .to_string();

        let request_id = context.request_id().map(|s| s.to_string());
        let idempotency_key = context.idempotency_key().map(|s| s.to_string());
        let result = crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "place_order",
            context.idempotency_key(),
            || async move {
                orders::handlers::place_order(
                    order,
                    user_id,
                    request_id.as_deref(),
                    idempotency_key.as_deref(),
                )
                .await
            },
        )
        .await;
        let reason = result.as_ref().err().map(|e| {
            let s = e.to_string();
            if s.contains("Insufficient stock") || s.contains("inventory") {
                "insufficient_stock"
            } else if s.contains("Unavailable") || s.contains("idempotency") {
                "idempotency"
            } else {
                "error"
            }
        });
        crate::metrics::record_place_order_total(result.is_ok(), reason);
        result.map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn create_order_details(
        order_details: NewOrderDetails,
    ) -> FieldResult<Vec<OrderDetails>> {
        order_details::handlers::create_order_detail(order_details)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_order_detail(
        order_detail: OrderDetailsMutation,
    ) -> FieldResult<Vec<OrderDetails>> {
        order_details::handlers::update_order_detail(order_detail)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_order(order_id: String) -> FieldResult<Vec<Order>> {
        orders::handlers::delete_order(order_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_order(order: OrderMutation) -> FieldResult<Vec<Order>> {
        orders::handlers::update_order(order)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn add_wishlist_item(wishlist: NewWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::add_wishlist_item(wishlist)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_wishlist_item(delete: DeleteWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::delete_wishlist_item(delete)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Country
    #[instrument(err, ret)]
    async fn create_country(country: NewCountry) -> FieldResult<Vec<Country>> {
        country::handlers::create_country(country)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_country(country_id: String) -> FieldResult<Vec<Country>> {
        country::handlers::delete_country(country_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // State
    #[instrument(err, ret)]
    async fn create_state(state: NewState) -> FieldResult<Vec<State>> {
        state::handlers::create_state(state)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_state(state_id: String) -> FieldResult<Vec<State>> {
        state::handlers::delete_state(state_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn create_payment_intent(input: NewPaymentIntent) -> FieldResult<Vec<PaymentIntent>> {
        payment_intents::handlers::create_payment_intent(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn capture_payment(
        context: &Context,
        input: CapturePayment,
    ) -> FieldResult<Vec<PaymentIntent>> {
        let request_id = context.request_id().map(|s| s.to_string());
        let result = crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "capture_payment",
            context.idempotency_key(),
            || async move {
                payment_intents::handlers::capture_payment(input, request_id.as_deref()).await
            },
        )
        .await;
        crate::metrics::record_capture_payment_total(result.is_ok());
        result.map_err(|e| e.into_field_error())
    }

    // ProductImage
    #[instrument(err, ret)]
    async fn delete_product_image(image_id: String) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::delete_product_image(image_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_image(
        product_image: ProductImageMutation,
    ) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::update_product_image(product_image)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipments
    #[instrument(err, ret)]
    async fn create_shipment(input: NewShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::create_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipment(input: UpdateShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::update_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Coupons
    #[instrument(err, ret)]
    async fn apply_coupon(input: ApplyCoupon) -> FieldResult<Vec<Coupon>> {
        coupons::handlers::apply_coupon(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Reviews
    #[instrument(err, ret)]
    async fn create_review(input: NewReview) -> FieldResult<Vec<Review>> {
        reviews::handlers::create_review(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_review(input: ReviewMutation) -> FieldResult<Vec<Review>> {
        reviews::handlers::update_review(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_review(review_id: String) -> FieldResult<Vec<Review>> {
        reviews::handlers::delete_review(review_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory
    #[instrument(err, ret)]
    async fn create_inventory_item(input: NewInventoryItem) -> FieldResult<Vec<InventoryItem>> {
        inventory::handlers::create_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_inventory_item(
        input: InventoryItemMutation,
    ) -> FieldResult<Vec<InventoryItem>> {
        inventory::handlers::update_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_inventory_item(inventory_id: String) -> FieldResult<Vec<InventoryItem>> {
        inventory::handlers::delete_inventory_item(inventory_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Discounts
    #[instrument(err, ret)]
    async fn create_discount(input: NewDiscount) -> FieldResult<Vec<Discount>> {
        discounts::handlers::create_discount(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_discount(input: DiscountMutation) -> FieldResult<Vec<Discount>> {
        discounts::handlers::update_discount(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_discount(discount_id: String) -> FieldResult<Vec<Discount>> {
        discounts::handlers::delete_discount(discount_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping methods
    #[instrument(err, ret)]
    async fn create_shipping_method(input: NewShippingMethod) -> FieldResult<Vec<ShippingMethod>> {
        shipping_methods::handlers::create_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipping_method(
        input: ShippingMethodMutation,
    ) -> FieldResult<Vec<ShippingMethod>> {
        shipping_methods::handlers::update_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_shipping_method(method_id: String) -> FieldResult<Vec<ShippingMethod>> {
        shipping_methods::handlers::delete_shipping_method(method_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping zones
    #[instrument(err, ret)]
    async fn create_shipping_zone(input: NewShippingZone) -> FieldResult<Vec<ShippingZone>> {
        shipping_zones::handlers::create_shipping_zone(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipping_zone(input: ShippingZoneMutation) -> FieldResult<Vec<ShippingZone>> {
        shipping_zones::handlers::update_shipping_zone(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_shipping_zone(zone_id: String) -> FieldResult<Vec<ShippingZone>> {
        shipping_zones::handlers::delete_shipping_zone(zone_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping addresses
    #[instrument(err, ret)]
    async fn create_shipping_address(
        input: NewShippingAddress,
    ) -> FieldResult<Vec<ShippingAddress>> {
        shipping_addresses::handlers::create_shipping_address(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipping_address(
        input: ShippingAddressMutation,
    ) -> FieldResult<Vec<ShippingAddress>> {
        shipping_addresses::handlers::update_shipping_address(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_shipping_address(
        shipping_address_id: String,
    ) -> FieldResult<Vec<ShippingAddress>> {
        shipping_addresses::handlers::delete_shipping_address(shipping_address_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Order Events
    #[instrument(err, ret)]
    async fn create_order_event(input: NewOrderEvent) -> FieldResult<Vec<OrderEvent>> {
        order_events::handlers::create_order_event(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product Images — R2 confirm upload
    #[instrument(err, ret)]
    async fn confirm_image_upload(input: ConfirmImageUpload) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::confirm_image_upload(input)
            .await
            .map_err(|e| e.into_field_error())
    }
}
