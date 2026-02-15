use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::users;
use proto::proto::core::{SearchUserRequest, UserResponse, UsersResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_user(
    txn: &DatabaseTransaction,
    request: Request<SearchUserRequest>,
) -> Result<Response<UsersResponse>, Status> {
    let req = request.into_inner();

    let mut query = users::Entity::find();
    if req.user_id != 0 {
        query = query.filter(users::Column::UserId.eq(req.user_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| UserResponse {
                    user_id: m.user_id,
                    username: m.username,
                    email: m.email,
                    full_name: m.full_name,
                    address: m.address,
                    phone: m.phone,
                    create_date: m.create_date.to_rfc3339(),
                })
                .collect();
            Ok(Response::new(UsersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
