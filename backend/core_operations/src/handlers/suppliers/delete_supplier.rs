use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::suppliers;
use proto::proto::core::{DeleteSupplierRequest, SupplierResponse, SuppliersResponse};
use sea_orm::{DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn delete_supplier(
    txn: &DatabaseTransaction,
    request: Request<DeleteSupplierRequest>,
) -> Result<Response<SuppliersResponse>, Status> {
    let req = request.into_inner();

    let found = suppliers::Entity::find_by_id(req.supplier_id)
        .one(txn)
        .await;

    match found {
        Ok(Some(model)) => {
            match suppliers::Entity::delete_by_id(req.supplier_id)
                .exec(txn)
                .await
            {
                Ok(_) => Ok(Response::new(SuppliersResponse {
                    items: vec![SupplierResponse {
                        supplier_id: model.supplier_id,
                        name: model.name.unwrap_or_default(),
                        contact_info: model.contact_info.unwrap_or_default(),
                        address: model.address.unwrap_or_default(),
                    }],
                })),
                Err(e) => Err(map_db_error_to_status(e)),
            }
        }
        Ok(None) => Err(Status::not_found(format!(
            "Supplier with ID {} not found",
            req.supplier_id
        ))),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
