use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::suppliers;
use proto::proto::core::{CreateSupplierRequest, SupplierResponse, SuppliersResponse};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseTransaction};
use tonic::{Request, Response, Status};

pub async fn create_supplier(
    txn: &DatabaseTransaction,
    request: Request<CreateSupplierRequest>,
) -> Result<Response<SuppliersResponse>, Status> {
    let req = request.into_inner();
    let model = suppliers::ActiveModel {
        supplier_id: ActiveValue::NotSet,
        name: ActiveValue::Set(Some(req.name)),
        contact_info: ActiveValue::Set(Some(req.contact_info)),
        address: ActiveValue::Set(Some(req.address)),
    };

    match model.insert(txn).await {
        Ok(inserted) => Ok(Response::new(SuppliersResponse {
            items: vec![SupplierResponse {
                supplier_id: inserted.supplier_id,
                name: inserted.name.unwrap_or_default(),
                contact_info: inserted.contact_info.unwrap_or_default(),
                address: inserted.address.unwrap_or_default(),
            }],
        })),
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
