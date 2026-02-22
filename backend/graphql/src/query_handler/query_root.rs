use super::Context;
use crate::resolvers::{
    cart::{self, schema::Cart},
    coupons::{self, schema::{Coupon, ValidateCoupon}},
    reviews::{self, schema::{Review, SearchReview}},
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
        schema::{ProductImage, SearchProductImage},
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

#[juniper::graphql_object(Context = Context)]
impl QueryRoot {
    // Cart
    #[instrument(err, ret)]
    async fn get_cart_items(
        user_id: Option<String>,
        session_id: Option<String>,
    ) -> FieldResult<Vec<Cart>> {
        cart::handlers::get_cart_items(user_id, session_id)
            .await
            .map_err(|e| e.into())
    }

    // Product
    #[instrument(err, ret)]
    async fn search_product(search: SearchProduct) -> FieldResult<Vec<Product>> {
        product::handlers::search_product(search)
            .await
            .map_err(|e| e.into())
    }

    // ProductImages
    #[instrument(err, ret)]
    async fn search_product_image(search: SearchProductImage) -> FieldResult<Vec<ProductImage>> {
        product_images::handlers::search_product_image(search)
            .await
            .map_err(|e| e.into())
    }

    // Category
    #[instrument(err, ret)]
    async fn search_category(search: SearchCategory) -> FieldResult<Vec<Category>> {
        category::handlers::search_category(search)
            .await
            .map_err(|e| e.into())
    }

    // Order
    #[instrument(err, ret)]
    async fn search_order(search: SearchOrder) -> FieldResult<Vec<Order>> {
        orders::handlers::search_order(search)
            .await
            .map_err(|e| e.into())
    }

    // Wishlist
    #[instrument(err, ret)]
    async fn search_wishlist_item(search: SearchWishlistItem) -> FieldResult<Vec<WishlistItem>> {
        wishlist::handlers::search_wishlist_item(search)
            .await
            .map_err(|e| e.into())
    }

    // Country
    #[instrument(err, ret)]
    async fn search_country(search: SearchCountry) -> FieldResult<Vec<Country>> {
        country::handlers::search_country(search)
            .await
            .map_err(|e| e.into())
    }

    // State
    #[instrument(err, ret)]
    async fn search_state(search: SearchState) -> FieldResult<Vec<State>> {
        state::handlers::search_state(search)
            .await
            .map_err(|e| e.into())
    }

    // PaymentIntents
    #[instrument(err, ret)]
    async fn get_payment_intent(input: GetPaymentIntent) -> FieldResult<Vec<PaymentIntent>> {
        payment_intents::handlers::get_payment_intent(input)
            .await
            .map_err(|e| e.into())
    }

    // Shipments
    #[instrument(err, ret)]
    async fn get_shipment(input: GetShipment) -> FieldResult<Vec<Shipment>> {
        shipments::handlers::get_shipment(input)
            .await
            .map_err(|e| e.into())
    }

    // Coupons
    #[instrument(err, ret)]
    async fn validate_coupon(input: ValidateCoupon) -> FieldResult<Vec<Coupon>> {
        coupons::handlers::validate_coupon(input)
            .await
            .map_err(|e| e.into())
    }

    // Reviews
    #[instrument(err, ret)]
    async fn search_review(input: SearchReview) -> FieldResult<Vec<Review>> {
        reviews::handlers::search_review(input)
            .await
            .map_err(|e| e.into())
    }
}
