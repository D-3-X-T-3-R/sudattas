use core_db_entities::{get_db, CoreDatabaseConnection};
use proto::proto::core::{
    grpc_services_server::GrpcServices, CartItemsResponse, CreateCartItemRequest,
    CreateProductRequest, DeleteCartItemRequest, DeleteProductRequest, GetCartItemsRequest,
    ProductsResponse, SearchProductRequest, UpdateCartItemRequest, UpdateProductRequest,
};
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
    // Cart
    async fn create_cart_item(
        &self,
        request: Request<CreateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        handlers::cart::create_cart_item(self.db.as_ref().unwrap(), request).await
    }

    async fn get_cart_items(
        &self,
        request: Request<GetCartItemsRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        handlers::cart::get_cart_items(self.db.as_ref().unwrap(), request).await
    }

    async fn update_cart_item(
        &self,
        request: Request<UpdateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        handlers::cart::update_cart_item(self.db.as_ref().unwrap(), request).await
    }

    async fn delete_cart_item(
        &self,
        request: Request<DeleteCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        handlers::cart::delete_cart_item(self.db.as_ref().unwrap(), request).await
    }

    // Product
    async fn create_product(
        &self,
        request: Request<CreateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        handlers::products::create_product(self.db.as_ref().unwrap(), request).await
    }

    async fn search_product(
        &self,
        request: Request<SearchProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        handlers::products::search_product(self.db.as_ref().unwrap(), request).await
    }

    async fn delete_product(
        &self,
        request: Request<DeleteProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        handlers::products::delete_product(self.db.as_ref().unwrap(), request).await
    }

    async fn update_product(
        &self,
        request: Request<UpdateProductRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        handlers::products::update_product(self.db.as_ref().unwrap(), request).await
    }
}
