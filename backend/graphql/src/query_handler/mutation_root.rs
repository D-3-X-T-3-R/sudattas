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
    colors::{
        self,
        schema::{Color, ColorMutation, DeleteColorInput, NewColor, SearchColorInput},
    },
    coupons::{
        self,
        schema::{ApplyCoupon, Coupon, CreateCouponInput, UpdateCouponInput},
    },
    event_logs::{
        self,
        schema::{
            DeleteEventLogInput, EventLog, EventLogMutation, NewEventLog, SearchEventLogInput,
        },
    },
    fabrics::{
        self,
        schema::{DeleteFabricInput, Fabric, FabricMutation, NewFabric, SearchFabricInput},
    },
    inventory::{
        self,
        schema::{InventoryItem, InventoryItemMutation, NewInventoryItem},
    },
    inventory_logs::{
        self,
        schema::{DeleteInventoryLogInput, InventoryLog, InventoryLogMutation, NewInventoryLog},
    },
    newsletter_subscribers::{
        self,
        schema::{
            DeleteNewsletterSubscriberInput, NewNewsletterSubscriber, NewsletterSubscriber,
            NewsletterSubscriberMutation, SearchNewsletterSubscriberInput,
        },
    },
    occasions::{
        self,
        schema::{
            DeleteOccasionInput, NewOccasion, Occasion, OccasionMutation, SearchOccasionInput,
        },
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
        schema::{
            AdminMarkOrderDeliveredInput, AdminMarkOrderShippedInput, CreateOrderInput, NewOrder,
            Order, OrderMutation,
        },
    },
    payment_intents::{
        self,
        schema::{
            CapturePayment, NewPaymentIntent, PaymentIntent, VerifyRazorpayPaymentInput,
            VerifyRazorpayPaymentResult,
        },
    },
    product::{
        self,
        schema::{NewProduct, Product, ProductMutation},
    },
    product_attribute_mappings::{
        self,
        schema::{
            DeleteProductAttributeMappingInput, NewProductAttributeMapping,
            ProductAttributeMapping, SearchProductAttributeMappingInput,
        },
    },
    product_attributes::{
        self,
        schema::{
            DeleteProductAttributeInput, NewProductAttribute, ProductAttribute,
            ProductAttributeMutation, SearchProductAttributeInput,
        },
    },
    product_images::{
        self,
        schema::{ConfirmImageUpload, ProductImage, ProductImageMutation},
    },
    product_variants::{
        self,
        schema::{
            DeleteProductVariantInput, NewProductVariant, ProductVariant, ProductVariantMutation,
        },
    },
    refunds::{
        self,
        schema::{NewRefund, Refund, ResolveNeedsReviewInput},
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
    sizes::{
        self,
        schema::{DeleteSizeInput, NewSize, SearchSizeInput, Size, SizeMutation},
    },
    transactions::{
        self,
        schema::{
            DeleteTransactionInput, NewTransaction, SearchTransactionInput, Transaction,
            TransactionMutation,
        },
    },
    user_activities::{
        self,
        schema::{
            DeleteUserActivityInput, NewUserActivity, SearchUserActivityInput, UserActivity,
            UserActivityMutation,
        },
    },
    user_roles::{
        self,
        schema::{
            DeleteUserRoleInput, NewUserRole, SearchUserRoleInput, UserRole, UserRoleMutation,
        },
    },
    users::{
        self,
        schema::{DeleteUserInput, NewUser, RecordSecurityAuditEventInput, UpdateUserInput, User},
    },
    weaves::{
        self,
        schema::{DeleteWeaveInput, NewWeave, SearchWeaveInput, Weave, WeaveMutation},
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

    /// P2 Abandoned cart: enqueue abandoned-cart events (typically from a cron/scheduler).
    /// Returns the number of events enqueued.
    #[instrument(err, ret)]
    async fn enqueue_abandoned_cart(delay_hours: Option<String>) -> FieldResult<i32> {
        let resp = cart::handlers::enqueue_abandoned_cart(delay_hours)
            .await
            .map_err(|e| e.into_field_error())?;
        Ok(resp.enqueued_count)
    }

    // Users
    #[instrument(err, ret)]
    async fn create_user(input: NewUser) -> FieldResult<Vec<User>> {
        users::handlers::create_user(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_user(input: UpdateUserInput) -> FieldResult<Vec<User>> {
        users::handlers::update_user(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_user(input: DeleteUserInput) -> FieldResult<Vec<User>> {
        users::handlers::delete_user(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// P2 security audit hook (e.g. secrets rotation).
    #[instrument(err, ret)]
    async fn record_security_audit_event(
        input: RecordSecurityAuditEventInput,
    ) -> FieldResult<bool> {
        users::handlers::record_security_audit_event(input)
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

    /// Low-level admin order creation (bypasses high-level checkout flow).
    #[instrument(err, ret)]
    async fn create_order_admin(input: CreateOrderInput) -> FieldResult<Vec<Order>> {
        orders::handlers::create_order_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: mark order shipped (creates shipment and updates status with enforced transitions).
    #[instrument(err, ret)]
    async fn admin_mark_order_shipped(input: AdminMarkOrderShippedInput) -> FieldResult<bool> {
        orders::handlers::admin_mark_order_shipped(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: mark order delivered with enforced transitions.
    #[instrument(err, ret)]
    async fn admin_mark_order_delivered(input: AdminMarkOrderDeliveredInput) -> FieldResult<bool> {
        orders::handlers::admin_mark_order_delivered(input)
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

    #[instrument(err, ret)]
    async fn verify_razorpay_payment(
        input: VerifyRazorpayPaymentInput,
    ) -> FieldResult<VerifyRazorpayPaymentResult> {
        payment_intents::handlers::verify_razorpay_payment(input)
            .await
            .map_err(|e| e.into_field_error())
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

    /// Admin: create a coupon.
    #[instrument(err, ret)]
    async fn create_coupon_admin(input: CreateCouponInput) -> FieldResult<bool> {
        coupons::handlers::create_coupon_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: update a coupon.
    #[instrument(err, ret)]
    async fn update_coupon_admin(input: UpdateCouponInput) -> FieldResult<bool> {
        coupons::handlers::update_coupon_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // User roles
    #[instrument(err, ret)]
    async fn create_user_role(input: NewUserRole) -> FieldResult<Vec<UserRole>> {
        user_roles::handlers::create_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_user_role(input: SearchUserRoleInput) -> FieldResult<Vec<UserRole>> {
        user_roles::handlers::search_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_user_role(input: UserRoleMutation) -> FieldResult<Vec<UserRole>> {
        user_roles::handlers::update_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_user_role(input: DeleteUserRoleInput) -> FieldResult<Vec<UserRole>> {
        user_roles::handlers::delete_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Transactions
    #[instrument(err, ret)]
    async fn create_transaction(input: NewTransaction) -> FieldResult<Vec<Transaction>> {
        transactions::handlers::create_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_transaction(input: SearchTransactionInput) -> FieldResult<Vec<Transaction>> {
        transactions::handlers::search_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_transaction(input: TransactionMutation) -> FieldResult<Vec<Transaction>> {
        transactions::handlers::update_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_transaction(input: DeleteTransactionInput) -> FieldResult<Vec<Transaction>> {
        transactions::handlers::delete_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Newsletter subscribers
    #[instrument(err, ret)]
    async fn create_newsletter_subscriber(
        input: NewNewsletterSubscriber,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        newsletter_subscribers::handlers::create_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_newsletter_subscriber(
        input: SearchNewsletterSubscriberInput,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        newsletter_subscribers::handlers::search_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_newsletter_subscriber(
        input: NewsletterSubscriberMutation,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        newsletter_subscribers::handlers::update_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_newsletter_subscriber(
        input: DeleteNewsletterSubscriberInput,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        newsletter_subscribers::handlers::delete_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Sizes
    #[instrument(err, ret)]
    async fn create_size(input: NewSize) -> FieldResult<Vec<Size>> {
        sizes::handlers::create_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_size(input: SearchSizeInput) -> FieldResult<Vec<Size>> {
        sizes::handlers::search_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_size(input: SizeMutation) -> FieldResult<Vec<Size>> {
        sizes::handlers::update_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_size(input: DeleteSizeInput) -> FieldResult<Vec<Size>> {
        sizes::handlers::delete_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Fabrics
    #[instrument(err, ret)]
    async fn create_fabric(input: NewFabric) -> FieldResult<Vec<Fabric>> {
        fabrics::handlers::create_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_fabric(input: SearchFabricInput) -> FieldResult<Vec<Fabric>> {
        fabrics::handlers::search_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_fabric(input: FabricMutation) -> FieldResult<Vec<Fabric>> {
        fabrics::handlers::update_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_fabric(input: DeleteFabricInput) -> FieldResult<Vec<Fabric>> {
        fabrics::handlers::delete_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Weaves
    #[instrument(err, ret)]
    async fn create_weave(input: NewWeave) -> FieldResult<Vec<Weave>> {
        weaves::handlers::create_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_weave(input: SearchWeaveInput) -> FieldResult<Vec<Weave>> {
        weaves::handlers::search_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_weave(input: WeaveMutation) -> FieldResult<Vec<Weave>> {
        weaves::handlers::update_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_weave(input: DeleteWeaveInput) -> FieldResult<Vec<Weave>> {
        weaves::handlers::delete_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Occasions
    #[instrument(err, ret)]
    async fn create_occasion(input: NewOccasion) -> FieldResult<Vec<Occasion>> {
        occasions::handlers::create_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_occasion(input: SearchOccasionInput) -> FieldResult<Vec<Occasion>> {
        occasions::handlers::search_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_occasion(input: OccasionMutation) -> FieldResult<Vec<Occasion>> {
        occasions::handlers::update_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_occasion(input: DeleteOccasionInput) -> FieldResult<Vec<Occasion>> {
        occasions::handlers::delete_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Colors
    #[instrument(err, ret)]
    async fn create_color(input: NewColor) -> FieldResult<Vec<Color>> {
        colors::handlers::create_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_color(input: SearchColorInput) -> FieldResult<Vec<Color>> {
        colors::handlers::search_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_color(input: ColorMutation) -> FieldResult<Vec<Color>> {
        colors::handlers::update_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_color(input: DeleteColorInput) -> FieldResult<Vec<Color>> {
        colors::handlers::delete_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Event logs
    #[instrument(err, ret)]
    async fn create_event_log(input: NewEventLog) -> FieldResult<Vec<EventLog>> {
        event_logs::handlers::create_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_event_log(input: SearchEventLogInput) -> FieldResult<Vec<EventLog>> {
        event_logs::handlers::search_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_event_log(input: EventLogMutation) -> FieldResult<Vec<EventLog>> {
        event_logs::handlers::update_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_event_log(input: DeleteEventLogInput) -> FieldResult<Vec<EventLog>> {
        event_logs::handlers::delete_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // User activities
    #[instrument(err, ret)]
    async fn create_user_activity(input: NewUserActivity) -> FieldResult<Vec<UserActivity>> {
        user_activities::handlers::create_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_user_activity(
        input: SearchUserActivityInput,
    ) -> FieldResult<Vec<UserActivity>> {
        user_activities::handlers::search_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_user_activity(input: UserActivityMutation) -> FieldResult<Vec<UserActivity>> {
        user_activities::handlers::update_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_user_activity(
        input: DeleteUserActivityInput,
    ) -> FieldResult<Vec<UserActivity>> {
        user_activities::handlers::delete_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory logs
    #[instrument(err, ret)]
    async fn create_inventory_log(input: NewInventoryLog) -> FieldResult<Vec<InventoryLog>> {
        inventory_logs::handlers::create_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_inventory_log(input: InventoryLogMutation) -> FieldResult<Vec<InventoryLog>> {
        inventory_logs::handlers::update_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_inventory_log(
        input: DeleteInventoryLogInput,
    ) -> FieldResult<Vec<InventoryLog>> {
        inventory_logs::handlers::delete_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product attribute mappings
    #[instrument(err, ret)]
    async fn create_product_attribute_mapping(
        input: NewProductAttributeMapping,
    ) -> FieldResult<Vec<ProductAttributeMapping>> {
        product_attribute_mappings::handlers::create_product_attribute_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_product_attribute_mapping(
        input: SearchProductAttributeMappingInput,
    ) -> FieldResult<Vec<ProductAttributeMapping>> {
        product_attribute_mappings::handlers::search_product_attribute_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_attribute_mapping(
        input: DeleteProductAttributeMappingInput,
    ) -> FieldResult<Vec<ProductAttributeMapping>> {
        product_attribute_mappings::handlers::delete_product_attribute_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Refunds
    #[instrument(err, ret)]
    async fn create_refund(input: NewRefund) -> FieldResult<Vec<Refund>> {
        refunds::handlers::create_refund(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Resolve NeedsReview manually (paid / cancelled / refunded).
    #[instrument(err, ret)]
    async fn resolve_needs_review(input: ResolveNeedsReviewInput) -> FieldResult<bool> {
        refunds::handlers::resolve_needs_review(input)
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

    /// Admin review moderation: approve/reject a review.
    #[instrument(err, ret)]
    async fn admin_update_review_status(
        input: crate::resolvers::reviews::schema::AdminUpdateReviewStatusInput,
    ) -> FieldResult<bool> {
        reviews::handlers::admin_update_review_status(input)
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

    // Product variants
    #[instrument(err, ret)]
    async fn create_product_variant(input: NewProductVariant) -> FieldResult<Vec<ProductVariant>> {
        product_variants::handlers::create_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_variant(
        input: ProductVariantMutation,
    ) -> FieldResult<Vec<ProductVariant>> {
        product_variants::handlers::update_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_variant(
        input: DeleteProductVariantInput,
    ) -> FieldResult<Vec<ProductVariant>> {
        product_variants::handlers::delete_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product attributes
    #[instrument(err, ret)]
    async fn create_product_attribute(
        input: NewProductAttribute,
    ) -> FieldResult<Vec<ProductAttribute>> {
        product_attributes::handlers::create_product_attribute(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_product_attribute(
        input: SearchProductAttributeInput,
    ) -> FieldResult<Vec<ProductAttribute>> {
        product_attributes::handlers::search_product_attribute(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_attribute(
        input: ProductAttributeMutation,
    ) -> FieldResult<Vec<ProductAttribute>> {
        product_attributes::handlers::update_product_attribute(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_attribute(
        input: DeleteProductAttributeInput,
    ) -> FieldResult<Vec<ProductAttribute>> {
        product_attributes::handlers::delete_product_attribute(input)
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
