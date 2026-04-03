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
    product_images::{
        self,
        schema::{ConfirmImageUpload, ProductImage, ProductImageMutation, SyncProductImagesInput},
    },
    product_mood_mappings::{
        self,
        schema::{
            DeleteProductMoodMappingInput, NewProductMoodMapping, ProductMoodMapping,
            SearchProductMoodMappingInput,
        },
    },
    product_moods::{
        self,
        schema::{DeleteProductMoodInput, NewProductMood, ProductMood, ProductMoodMutation},
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
use tracing::{info, warn};

pub struct MutationRoot;

fn require_jwt(context: &Context) -> Result<&str, juniper::FieldError> {
    context.jwt_user_id().ok_or_else(|| {
        juniper::FieldError::new("Login required for this operation", juniper::Value::null())
    })
}

fn require_admin(context: &Context) -> Result<(), juniper::FieldError> {
    if context.is_admin() {
        Ok(())
    } else {
        crate::metrics::record_admin_authz_denied_total();
        Err(juniper::FieldError::new(
            "Admin authorization required",
            juniper::Value::null(),
        ))
    }
}

async fn ensure_customer_owns_shipping_address(
    context: &Context,
    shipping_address_id: &str,
) -> Result<(), juniper::FieldError> {
    if context.is_admin() {
        return Ok(());
    }
    let uid = require_jwt(context)?.to_string();
    let rows = shipping_addresses::handlers::get_shipping_addresses()
        .await
        .map_err(|e| e.into_field_error())?;

    let belongs_to_user = rows.into_iter().any(|row| {
        row.shipping_address_id == shipping_address_id && row.user_id.as_deref() == Some(uid.as_str())
    });
    if belongs_to_user {
        Ok(())
    } else {
        Err(juniper::FieldError::new(
            "Shipping address not found for current user",
            juniper::Value::null(),
        ))
    }
}

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    // Cart
    #[instrument(err, ret)]
    async fn add_cart_item(context: &Context, cart_item: NewCart) -> FieldResult<Vec<Cart>> {
        crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "add_cart_item",
            context.idempotency_key(),
            || async move { cart::handlers::add_cart_item(cart_item).await },
        )
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
    async fn delete_cart_item(context: &Context, delete: DeleteCartItem) -> FieldResult<Vec<Cart>> {
        crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "delete_cart_item",
            context.idempotency_key(),
            || async move { cart::handlers::delete_cart_item(delete).await },
        )
        .await
        .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_cart_item(context: &Context, cart_item: CartMutation) -> FieldResult<Vec<Cart>> {
        crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "update_cart_item",
            context.idempotency_key(),
            || async move { cart::handlers::update_cart_item(cart_item).await },
        )
        .await
        .map_err(|e| e.into_field_error())
    }

    // Product
    #[instrument(err, ret)]
    async fn create_product(context: &Context, product: NewProduct) -> FieldResult<Vec<Product>> {
        require_admin(context)?;
        product::handlers::create_product(product)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product(context: &Context, product_id: String) -> FieldResult<Vec<Product>> {
        require_admin(context)?;
        product::handlers::delete_product(product_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product(
        context: &Context,
        product: ProductMutation,
    ) -> FieldResult<Vec<Product>> {
        require_admin(context)?;
        product::handlers::update_product(product)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Category
    #[instrument(err, ret)]
    async fn create_category(
        context: &Context,
        category: NewCategory,
    ) -> FieldResult<Vec<Category>> {
        require_admin(context)?;
        category::handlers::create_category(category)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_category(context: &Context, category_id: String) -> FieldResult<Vec<Category>> {
        require_admin(context)?;
        category::handlers::delete_category(category_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_category(
        context: &Context,
        category: CategoryMutation,
    ) -> FieldResult<Vec<Category>> {
        require_admin(context)?;
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
        let user_id_for_grpc = user_id.clone();
        info!(
            request_id = ?context.request_id(),
            client_action = ?context.client_action(),
            user_id = %user_id,
            auth_mode = %context.auth_mode(),
            shipping_address_id = %order.shipping_address_id,
            has_coupon_code = order.coupon_code.as_ref().map(|c| !c.trim().is_empty()).unwrap_or(false),
            has_idempotency_key = context.idempotency_key().is_some(),
            "checkout.place_order.start"
        );
        let result = crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "place_order",
            context.idempotency_key(),
            || async move {
                orders::handlers::place_order(
                    order,
                    user_id_for_grpc,
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
        if let Err(ref e) = result {
            warn!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                user_id = %user_id,
                auth_mode = %context.auth_mode(),
                has_idempotency_key = context.idempotency_key().is_some(),
                error = %e,
                "checkout.place_order.failed"
            );
        } else {
            info!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                user_id = %user_id,
                auth_mode = %context.auth_mode(),
                "checkout.place_order.success"
            );
        }
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
    async fn delete_order(context: &Context, order_id: String) -> FieldResult<Vec<Order>> {
        require_admin(context)?;
        orders::handlers::delete_order(order_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_order(context: &Context, order: OrderMutation) -> FieldResult<Vec<Order>> {
        require_admin(context)?;
        orders::handlers::update_order(order)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Low-level admin order creation (bypasses high-level checkout flow).
    #[instrument(err, ret)]
    async fn create_order_admin(
        context: &Context,
        input: CreateOrderInput,
    ) -> FieldResult<Vec<Order>> {
        require_admin(context)?;
        orders::handlers::create_order_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: mark order shipped (creates shipment and updates status with enforced transitions).
    #[instrument(err, ret)]
    async fn admin_mark_order_shipped(
        context: &Context,
        input: AdminMarkOrderShippedInput,
    ) -> FieldResult<bool> {
        require_admin(context)?;
        orders::handlers::admin_mark_order_shipped(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: mark order delivered with enforced transitions.
    #[instrument(err, ret)]
    async fn admin_mark_order_delivered(
        context: &Context,
        input: AdminMarkOrderDeliveredInput,
    ) -> FieldResult<bool> {
        require_admin(context)?;
        orders::handlers::admin_mark_order_delivered(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn add_wishlist_item(
        context: &Context,
        mut wishlist: NewWishlistItem,
    ) -> FieldResult<Vec<WishlistItem>> {
        if !context.is_admin() {
            wishlist.user_id = require_jwt(context)?.to_string();
        }
        wishlist::handlers::add_wishlist_item(wishlist)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_wishlist_item(
        context: &Context,
        mut delete: DeleteWishlistItem,
    ) -> FieldResult<Vec<WishlistItem>> {
        if !context.is_admin() {
            delete.user_id = require_jwt(context)?.to_string();
        }
        wishlist::handlers::delete_wishlist_item(delete)
            .await
            .map_err(|e| e.into_field_error())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn create_payment_intent(
        context: &Context,
        input: NewPaymentIntent,
    ) -> FieldResult<Vec<PaymentIntent>> {
        info!(
            request_id = ?context.request_id(),
            client_action = ?context.client_action(),
            auth_mode = %context.auth_mode(),
            user_id = %input.user_id,
            order_id = %input.order_id,
            amount_paise = %input.amount_paise,
            has_idempotency_key = context.idempotency_key().is_some(),
            "checkout.create_payment_intent.start"
        );
        let request_id = context.request_id().map(|s| s.to_string());
        let result = payment_intents::handlers::create_payment_intent(input, request_id.as_deref()).await;
        if let Err(ref e) = result {
            warn!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                auth_mode = %context.auth_mode(),
                error = %e,
                "checkout.create_payment_intent.failed"
            );
        } else {
            info!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                auth_mode = %context.auth_mode(),
                "checkout.create_payment_intent.success"
            );
        }
        result.map_err(|e| e.into_field_error())
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
        context: &Context,
        input: VerifyRazorpayPaymentInput,
    ) -> FieldResult<VerifyRazorpayPaymentResult> {
        info!(
            request_id = ?context.request_id(),
            client_action = ?context.client_action(),
            auth_mode = %context.auth_mode(),
            order_id = %input.order_id,
            has_idempotency_key = context.idempotency_key().is_some(),
            "checkout.verify_razorpay_payment.start"
        );
        let request_id = context.request_id().map(|s| s.to_string());
        let result = crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "verify_razorpay_payment",
            context.idempotency_key(),
            || async move {
                payment_intents::handlers::verify_razorpay_payment(input, request_id.as_deref()).await
            },
        )
        .await;
        if let Err(ref e) = result {
            warn!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                auth_mode = %context.auth_mode(),
                has_idempotency_key = context.idempotency_key().is_some(),
                error = %e,
                "checkout.verify_razorpay_payment.failed"
            );
        } else {
            info!(
                request_id = ?context.request_id(),
                client_action = ?context.client_action(),
                auth_mode = %context.auth_mode(),
                "checkout.verify_razorpay_payment.success"
            );
        }
        result.map_err(|e| e.into_field_error())
    }

    // ProductImage
    #[instrument(err, ret)]
    async fn delete_product_image(
        context: &Context,
        image_id: String,
    ) -> FieldResult<Vec<ProductImage>> {
        require_admin(context)?;
        product_images::handlers::delete_product_image(image_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_image(
        context: &Context,
        product_image: ProductImageMutation,
    ) -> FieldResult<Vec<ProductImage>> {
        require_admin(context)?;
        product_images::handlers::update_product_image(product_image)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipments
    #[instrument(err, ret)]
    async fn create_shipment(context: &Context, input: NewShipment) -> FieldResult<Vec<Shipment>> {
        require_admin(context)?;
        shipments::handlers::create_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipment(
        context: &Context,
        input: UpdateShipment,
    ) -> FieldResult<Vec<Shipment>> {
        require_admin(context)?;
        shipments::handlers::update_shipment(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Coupons
    #[instrument(err, ret)]
    async fn apply_coupon(context: &Context, input: ApplyCoupon) -> FieldResult<Vec<Coupon>> {
        crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "apply_coupon",
            context.idempotency_key(),
            || async move { coupons::handlers::apply_coupon(input).await },
        )
        .await
        .map_err(|e| e.into_field_error())
    }

    /// Admin: create a coupon.
    #[instrument(err, ret)]
    async fn create_coupon_admin(context: &Context, input: CreateCouponInput) -> FieldResult<bool> {
        require_admin(context)?;
        coupons::handlers::create_coupon_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin: update a coupon.
    #[instrument(err, ret)]
    async fn update_coupon_admin(context: &Context, input: UpdateCouponInput) -> FieldResult<bool> {
        require_admin(context)?;
        coupons::handlers::update_coupon_admin(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // User roles
    #[instrument(err, ret)]
    async fn create_user_role(context: &Context, input: NewUserRole) -> FieldResult<Vec<UserRole>> {
        require_admin(context)?;
        user_roles::handlers::create_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_user_role(
        context: &Context,
        input: SearchUserRoleInput,
    ) -> FieldResult<Vec<UserRole>> {
        require_admin(context)?;
        user_roles::handlers::search_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_user_role(
        context: &Context,
        input: UserRoleMutation,
    ) -> FieldResult<Vec<UserRole>> {
        require_admin(context)?;
        user_roles::handlers::update_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_user_role(
        context: &Context,
        input: DeleteUserRoleInput,
    ) -> FieldResult<Vec<UserRole>> {
        require_admin(context)?;
        user_roles::handlers::delete_user_role(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Transactions
    #[instrument(err, ret)]
    async fn create_transaction(
        context: &Context,
        input: NewTransaction,
    ) -> FieldResult<Vec<Transaction>> {
        require_admin(context)?;
        transactions::handlers::create_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_transaction(
        context: &Context,
        input: SearchTransactionInput,
    ) -> FieldResult<Vec<Transaction>> {
        require_admin(context)?;
        transactions::handlers::search_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_transaction(
        context: &Context,
        input: TransactionMutation,
    ) -> FieldResult<Vec<Transaction>> {
        require_admin(context)?;
        transactions::handlers::update_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_transaction(
        context: &Context,
        input: DeleteTransactionInput,
    ) -> FieldResult<Vec<Transaction>> {
        require_admin(context)?;
        transactions::handlers::delete_transaction(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Newsletter subscribers
    #[instrument(err, ret)]
    async fn create_newsletter_subscriber(
        context: &Context,
        input: NewNewsletterSubscriber,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        require_admin(context)?;
        newsletter_subscribers::handlers::create_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_newsletter_subscriber(
        context: &Context,
        input: SearchNewsletterSubscriberInput,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        require_admin(context)?;
        newsletter_subscribers::handlers::search_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_newsletter_subscriber(
        context: &Context,
        input: NewsletterSubscriberMutation,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        require_admin(context)?;
        newsletter_subscribers::handlers::update_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_newsletter_subscriber(
        context: &Context,
        input: DeleteNewsletterSubscriberInput,
    ) -> FieldResult<Vec<NewsletterSubscriber>> {
        require_admin(context)?;
        newsletter_subscribers::handlers::delete_newsletter_subscriber(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Sizes
    #[instrument(err, ret)]
    async fn create_size(context: &Context, input: NewSize) -> FieldResult<Vec<Size>> {
        require_admin(context)?;
        sizes::handlers::create_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_size(context: &Context, input: SearchSizeInput) -> FieldResult<Vec<Size>> {
        require_admin(context)?;
        sizes::handlers::search_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_size(context: &Context, input: SizeMutation) -> FieldResult<Vec<Size>> {
        require_admin(context)?;
        sizes::handlers::update_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_size(context: &Context, input: DeleteSizeInput) -> FieldResult<Vec<Size>> {
        require_admin(context)?;
        sizes::handlers::delete_size(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Fabrics
    #[instrument(err, ret)]
    async fn create_fabric(context: &Context, input: NewFabric) -> FieldResult<Vec<Fabric>> {
        require_admin(context)?;
        fabrics::handlers::create_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_fabric(
        context: &Context,
        input: SearchFabricInput,
    ) -> FieldResult<Vec<Fabric>> {
        require_admin(context)?;
        fabrics::handlers::search_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_fabric(context: &Context, input: FabricMutation) -> FieldResult<Vec<Fabric>> {
        require_admin(context)?;
        fabrics::handlers::update_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_fabric(
        context: &Context,
        input: DeleteFabricInput,
    ) -> FieldResult<Vec<Fabric>> {
        require_admin(context)?;
        fabrics::handlers::delete_fabric(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Weaves
    #[instrument(err, ret)]
    async fn create_weave(context: &Context, input: NewWeave) -> FieldResult<Vec<Weave>> {
        require_admin(context)?;
        weaves::handlers::create_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_weave(context: &Context, input: SearchWeaveInput) -> FieldResult<Vec<Weave>> {
        require_admin(context)?;
        weaves::handlers::search_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_weave(context: &Context, input: WeaveMutation) -> FieldResult<Vec<Weave>> {
        require_admin(context)?;
        weaves::handlers::update_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_weave(context: &Context, input: DeleteWeaveInput) -> FieldResult<Vec<Weave>> {
        require_admin(context)?;
        weaves::handlers::delete_weave(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Occasions
    #[instrument(err, ret)]
    async fn create_occasion(context: &Context, input: NewOccasion) -> FieldResult<Vec<Occasion>> {
        require_admin(context)?;
        occasions::handlers::create_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_occasion(
        context: &Context,
        input: SearchOccasionInput,
    ) -> FieldResult<Vec<Occasion>> {
        require_admin(context)?;
        occasions::handlers::search_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_occasion(
        context: &Context,
        input: OccasionMutation,
    ) -> FieldResult<Vec<Occasion>> {
        require_admin(context)?;
        occasions::handlers::update_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_occasion(
        context: &Context,
        input: DeleteOccasionInput,
    ) -> FieldResult<Vec<Occasion>> {
        require_admin(context)?;
        occasions::handlers::delete_occasion(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Colors
    #[instrument(err, ret)]
    async fn create_color(context: &Context, input: NewColor) -> FieldResult<Vec<Color>> {
        require_admin(context)?;
        colors::handlers::create_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_color(context: &Context, input: SearchColorInput) -> FieldResult<Vec<Color>> {
        require_admin(context)?;
        colors::handlers::search_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_color(context: &Context, input: ColorMutation) -> FieldResult<Vec<Color>> {
        require_admin(context)?;
        colors::handlers::update_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_color(context: &Context, input: DeleteColorInput) -> FieldResult<Vec<Color>> {
        require_admin(context)?;
        colors::handlers::delete_color(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Event logs
    #[instrument(err, ret)]
    async fn create_event_log(context: &Context, input: NewEventLog) -> FieldResult<Vec<EventLog>> {
        require_admin(context)?;
        event_logs::handlers::create_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_event_log(
        context: &Context,
        input: SearchEventLogInput,
    ) -> FieldResult<Vec<EventLog>> {
        require_admin(context)?;
        event_logs::handlers::search_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_event_log(
        context: &Context,
        input: EventLogMutation,
    ) -> FieldResult<Vec<EventLog>> {
        require_admin(context)?;
        event_logs::handlers::update_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_event_log(
        context: &Context,
        input: DeleteEventLogInput,
    ) -> FieldResult<Vec<EventLog>> {
        require_admin(context)?;
        event_logs::handlers::delete_event_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // User activities
    #[instrument(err, ret)]
    async fn create_user_activity(
        context: &Context,
        input: NewUserActivity,
    ) -> FieldResult<Vec<UserActivity>> {
        require_admin(context)?;
        user_activities::handlers::create_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_user_activity(
        context: &Context,
        input: SearchUserActivityInput,
    ) -> FieldResult<Vec<UserActivity>> {
        require_admin(context)?;
        user_activities::handlers::search_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_user_activity(
        context: &Context,
        input: UserActivityMutation,
    ) -> FieldResult<Vec<UserActivity>> {
        require_admin(context)?;
        user_activities::handlers::update_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_user_activity(
        context: &Context,
        input: DeleteUserActivityInput,
    ) -> FieldResult<Vec<UserActivity>> {
        require_admin(context)?;
        user_activities::handlers::delete_user_activity(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory logs
    #[instrument(err, ret)]
    async fn create_inventory_log(
        context: &Context,
        input: NewInventoryLog,
    ) -> FieldResult<Vec<InventoryLog>> {
        require_admin(context)?;
        inventory_logs::handlers::create_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_inventory_log(
        context: &Context,
        input: InventoryLogMutation,
    ) -> FieldResult<Vec<InventoryLog>> {
        require_admin(context)?;
        inventory_logs::handlers::update_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_inventory_log(
        context: &Context,
        input: DeleteInventoryLogInput,
    ) -> FieldResult<Vec<InventoryLog>> {
        require_admin(context)?;
        inventory_logs::handlers::delete_inventory_log(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product mood mappings
    #[instrument(err, ret)]
    async fn create_product_mood_mapping(
        context: &Context,
        input: NewProductMoodMapping,
    ) -> FieldResult<Vec<ProductMoodMapping>> {
        require_admin(context)?;
        product_mood_mappings::handlers::create_product_mood_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn search_product_mood_mapping(
        context: &Context,
        input: SearchProductMoodMappingInput,
    ) -> FieldResult<Vec<ProductMoodMapping>> {
        require_admin(context)?;
        product_mood_mappings::handlers::search_product_mood_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_mood_mapping(
        context: &Context,
        input: DeleteProductMoodMappingInput,
    ) -> FieldResult<Vec<ProductMoodMapping>> {
        require_admin(context)?;
        product_mood_mappings::handlers::delete_product_mood_mapping(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Refunds
    #[instrument(err, ret)]
    async fn create_refund(context: &Context, input: NewRefund) -> FieldResult<Vec<Refund>> {
        require_admin(context)?;
        refunds::handlers::create_refund(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Resolve NeedsReview manually (paid / cancelled / refunded).
    #[instrument(err, ret)]
    async fn resolve_needs_review(
        context: &Context,
        input: ResolveNeedsReviewInput,
    ) -> FieldResult<bool> {
        require_admin(context)?;
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
    async fn delete_review(context: &Context, review_id: String) -> FieldResult<Vec<Review>> {
        require_admin(context)?;
        reviews::handlers::delete_review(review_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    /// Admin review moderation: approve/reject a review.
    #[instrument(err, ret)]
    async fn admin_update_review_status(
        context: &Context,
        input: crate::resolvers::reviews::schema::AdminUpdateReviewStatusInput,
    ) -> FieldResult<bool> {
        require_admin(context)?;
        reviews::handlers::admin_update_review_status(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Inventory
    #[instrument(err, ret)]
    async fn create_inventory_item(
        context: &Context,
        input: NewInventoryItem,
    ) -> FieldResult<Vec<InventoryItem>> {
        require_admin(context)?;
        inventory::handlers::create_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_inventory_item(
        context: &Context,
        input: InventoryItemMutation,
    ) -> FieldResult<Vec<InventoryItem>> {
        require_admin(context)?;
        inventory::handlers::update_inventory_item(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_inventory_item(
        context: &Context,
        inventory_id: String,
    ) -> FieldResult<Vec<InventoryItem>> {
        require_admin(context)?;
        inventory::handlers::delete_inventory_item(inventory_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product variants
    #[instrument(err, ret)]
    async fn create_product_variant(
        context: &Context,
        input: NewProductVariant,
    ) -> FieldResult<Vec<ProductVariant>> {
        require_admin(context)?;
        product_variants::handlers::create_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_variant(
        context: &Context,
        input: ProductVariantMutation,
    ) -> FieldResult<Vec<ProductVariant>> {
        require_admin(context)?;
        product_variants::handlers::update_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_variant(
        context: &Context,
        input: DeleteProductVariantInput,
    ) -> FieldResult<Vec<ProductVariant>> {
        require_admin(context)?;
        product_variants::handlers::delete_product_variant(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product moods
    #[instrument(err, ret)]
    async fn create_product_mood(
        context: &Context,
        input: NewProductMood,
    ) -> FieldResult<Vec<ProductMood>> {
        require_admin(context)?;
        product_moods::handlers::create_product_mood(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_product_mood(
        context: &Context,
        input: ProductMoodMutation,
    ) -> FieldResult<Vec<ProductMood>> {
        require_admin(context)?;
        product_moods::handlers::update_product_mood(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_product_mood(
        context: &Context,
        input: DeleteProductMoodInput,
    ) -> FieldResult<Vec<ProductMood>> {
        require_admin(context)?;
        product_moods::handlers::delete_product_mood(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping methods
    #[instrument(err, ret)]
    async fn create_shipping_method(
        context: &Context,
        input: NewShippingMethod,
    ) -> FieldResult<Vec<ShippingMethod>> {
        require_admin(context)?;
        shipping_methods::handlers::create_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipping_method(
        context: &Context,
        input: ShippingMethodMutation,
    ) -> FieldResult<Vec<ShippingMethod>> {
        require_admin(context)?;
        shipping_methods::handlers::update_shipping_method(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_shipping_method(
        context: &Context,
        method_id: String,
    ) -> FieldResult<Vec<ShippingMethod>> {
        require_admin(context)?;
        shipping_methods::handlers::delete_shipping_method(method_id)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Shipping addresses
    #[instrument(err, ret)]
    async fn create_shipping_address(
        context: &Context,
        mut input: NewShippingAddress,
    ) -> FieldResult<Vec<ShippingAddress>> {
        let jwt_uid = require_jwt(context)?.to_string();
        if !context.is_admin() {
            input.user_id = Some(jwt_uid);
        }
        crate::idempotency::with_idempotency(
            context.redis_url.as_deref(),
            "create_shipping_address",
            context.idempotency_key(),
            || async move { shipping_addresses::handlers::create_shipping_address(input).await },
        )
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn update_shipping_address(
        context: &Context,
        mut input: ShippingAddressMutation,
    ) -> FieldResult<Vec<ShippingAddress>> {
        let jwt_uid = require_jwt(context)?.to_string();
        ensure_customer_owns_shipping_address(context, &input.shipping_address_id).await?;
        if !context.is_admin() {
            input.user_id = Some(jwt_uid);
        }
        shipping_addresses::handlers::update_shipping_address(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    #[instrument(err, ret)]
    async fn delete_shipping_address(
        context: &Context,
        shipping_address_id: String,
    ) -> FieldResult<Vec<ShippingAddress>> {
        let _ = require_jwt(context)?;
        ensure_customer_owns_shipping_address(context, &shipping_address_id).await?;
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
    async fn confirm_image_upload(
        context: &Context,
        input: ConfirmImageUpload,
    ) -> FieldResult<Vec<ProductImage>> {
        require_admin(context)?;
        product_images::handlers::confirm_image_upload(input)
            .await
            .map_err(|e| e.into_field_error())
    }

    // Product Images — sync order (update kept, bulk insert new, delete removed)
    #[instrument(err, ret)]
    async fn sync_product_images(
        context: &Context,
        input: SyncProductImagesInput,
    ) -> FieldResult<Vec<ProductImage>> {
        require_admin(context)?;
        product_images::handlers::sync_product_images(input)
            .await
            .map_err(|e| e.into_field_error())
    }
}
