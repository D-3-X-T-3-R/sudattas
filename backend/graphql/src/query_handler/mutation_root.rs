use super::Context;
use crate::resolvers::{
    cart::{
        self,
        schema::{Cart, CartMutation, DeleteCartItem, NewCart},
    },
    payment_intents::{
        self,
        schema::{CapturePayment, NewPaymentIntent, PaymentIntent},
    },
    shipments::{
        self,
        schema::{NewShipment, Shipment, UpdateShipment},
    },
    category::{
        self,
        schema::{Category, CategoryMutation, NewCategory},
    },
    country::{self, schema::{Country, NewCountry}},
    order_details::{
        self,
        schema::{NewOrderDetails, OrderDetails, OrderDetailsMutation},
    },
    orders::{
        self,
        schema::{NewOrder, Order, OrderMutation},
    },
    product::{
        self,
        schema::{NewProduct, Product, ProductMutation},
    },
    product_images::{
        self,
        schema::{NewProductImage, ProductImage, ProductImageMutation},
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

pub struct MutationRoot;

#[juniper::graphql_object(Context = Context)]
impl MutationRoot {
    // Cart
    #[instrument(err, ret)]
    async fn add_cart_item(cart_item: NewCart) -> FieldResult<Vec<Cart>> {
        cart::handlers::add_cart_item(cart_item)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_cart_item(delete: DeleteCartItem) -> FieldResult<Vec<Cart>> {
        cart::handlers::delete_cart_item(delete)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_cart_item(cart_item: CartMutation) -> FieldResult<Vec<Cart>> {
        cart::handlers::update_cart_item(cart_item)
            .await
            .map_err(|e| e.into())
    }

    // Product
    #[instrument(err, ret)]
    async fn create_product(product: NewProduct) -> FieldResult<Vec<Product>> {
        product::handlers::create_product(product)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_product(product_id: String) -> FieldResult<Vec<Product>> {
        product::handlers::delete_product(product_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_product(product: ProductMutation) -> FieldResult<Vec<Product>> {
        product::handlers::update_product(product)
            .await
            .map_err(|e| e.into())
    }

    // Category
    #[instrument(err, ret)]
    async fn create_category(category: NewCategory) -> FieldResult<Vec<Category>> {
        category::handlers::create_category(category)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_category(category_id: String) -> FieldResult<Vec<Category>> {
        category::handlers::delete_category(category_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_category(category: CategoryMutation) -> FieldResult<Vec<Category>> {
        category::handlers::update_category(category)
            .await
            .map_err(|e| e.into())
    }

    // Order
    #[instrument(err, ret)]
    async fn place_order(order: NewOrder) -> FieldResult<Vec<Order>> {
        orders::handlers::place_order(order)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn create_order_details(order_details: NewOrderDetails) -> FieldResult<Vec<OrderDetails>> {
        order_details::handlers::create_order_detail(order_details)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_order_detail(order_detail: OrderDetailsMutation) -> FieldResult<Vec<OrderDetails>> {
        order_details::handlers::update_order_detail(order_detail)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_order(order_id: String) -> FieldResult<Vec<Order>> {
        orders::handlers::delete_order(order_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_order(order: OrderMutation) -> FieldResult<Vec<Order>> {
        orders::handlers::update_order(order)
            .await
            .map_err(|e| e.into())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn add_wishlist_item(wishlist: NewWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::add_wishlist_item(wishlist)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_wishlist_item(delete: DeleteWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::delete_wishlist_item(delete)
            .await
            .map_err(|e| e.into())
    }

    // Country
    #[instrument(err, ret)]
    async fn create_country(country: NewCountry) -> FieldResult<Vec<Country>> {
        country::handlers::create_country(country)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_country(country_id: String) -> FieldResult<Vec<Country>> {
        country::handlers::delete_country(country_id)
            .await
            .map_err(|e| e.into())
    }

    // State
    #[instrument(err, ret)]
    async fn create_state(state: NewState) -> FieldResult<Vec<State>> {
        state::handlers::create_state(state)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_state(state_id: String) -> FieldResult<Vec<State>> {
        state::handlers::delete_state(state_id)
            .await
            .map_err(|e| e.into())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn create_payment_intent(input: NewPaymentIntent) -> FieldResult<Vec<PaymentIntent>> {
        payment_intents::handlers::create_payment_intent(input)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn capture_payment(input: CapturePayment) -> FieldResult<Vec<PaymentIntent>> {
        payment_intents::handlers::capture_payment(input)
            .await
            .map_err(|e| e.into())
    }

    // ProductImage
    #[instrument(err, ret)]
    async fn add_product_image(product_image: NewProductImage) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::add_product_image(product_image)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn delete_product_image(image_id: String) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::delete_product_image(image_id)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_product_image(
        product_image: ProductImageMutation,
    ) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::update_product_image(product_image)
            .await
            .map_err(|e| e.into())
    }

    // Shipments
    #[instrument(err, ret)]
    async fn create_shipment(input: NewShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::create_shipment(input)
            .await
            .map_err(|e| e.into())
    }

    #[instrument(err, ret)]
    async fn update_shipment(input: UpdateShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::update_shipment(input)
            .await
            .map_err(|e| e.into())
    }
}
