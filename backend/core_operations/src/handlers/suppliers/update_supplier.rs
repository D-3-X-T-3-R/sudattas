use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::suppliers;
use proto::proto::core::{SupplierResponse, SuppliersResponse, UpdateSupplierRequest};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction, EntityTrait};
use tonic::{Request, Response, Status};

pub async fn update_supplier(
    txn: &DatabaseTransaction,
    request: Request<UpdateSupplierRequest>,
) -> Result<Response<SuppliersResponse>, Status> {
    let req = request.into_inner();

    let existing = suppliers::Entity::find_by_id(req.supplier_id)
        .one(txn)
        .await
        .map_err(map_db_error_to_status)?
        .ok_or_else(|| {
            Status::not_found(format!("Supplier with ID {} not found", req.supplier_id))
        })?;

    let model = suppliers::ActiveModel {
        supplier_id: ActiveValue::Set(existing.supplier_id),
        name: ActiveValue::Set(req.name.or(existing.name)),
        contact_info: ActiveValue::Set(req.contact_info.or(existing.contact_info)),
        address: ActiveValue::Set(req.address.or(existing.address)),
    };

    match model.update(txn).await {
        Ok(updated) => Ok(Response::new(SuppliersResponse {
            items: vec![SupplierResponse {
                supplier_id: updated.supplier_id,
                name: updated.name.unwrap_or_default(),
                contact_info: updated.contact_info.unwrap_or_default(),
                address: updated.address.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
