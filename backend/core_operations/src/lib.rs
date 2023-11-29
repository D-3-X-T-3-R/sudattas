use core_db_entities::{get_db, CoreDatabaseConnection};
use proto::proto::core::{
    grpc_services_server::GrpcServices, CartItemsResponse, CreateCartItemRequest,
    DeleteCartItemRequest, ReadCartItemsRequest, UpdateCartItemRequest,
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
    async fn create_cart_item(
        &self,
        request: Request<CreateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        handlers::cart::create_cart_item(self.db.as_ref().unwrap(), request).await
    }

    async fn read_cart_items(
        &self,
        request: Request<ReadCartItemsRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        todo!()
    }

    async fn update_cart_item(
        &self,
        request: Request<UpdateCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        todo!()
    }

    async fn delete_cart_item(
        &self,
        request: Request<DeleteCartItemRequest>,
    ) -> Result<Response<CartItemsResponse>, Status> {
        todo!()
    }
}
