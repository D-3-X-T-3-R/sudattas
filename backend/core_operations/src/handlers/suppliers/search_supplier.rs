use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::suppliers;
use proto::proto::core::{SearchSupplierRequest, SupplierResponse, SuppliersResponse};
use sea_orm::{ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryTrait};
use tonic::{Request, Response, Status};

pub async fn search_supplier(
    txn: &DatabaseTransaction,
    request: Request<SearchSupplierRequest>,
) -> Result<Response<SuppliersResponse>, Status> {
    let req = request.into_inner();

    let mut query = suppliers::Entity::find();
    if req.supplier_id != 0 {
        query = query.filter(suppliers::Column::SupplierId.eq(req.supplier_id));
    }

    match query.all(txn).await {
        Ok(models) => {
            let items = models
                .into_iter()
                .map(|m| SupplierResponse {
                    supplier_id: m.supplier_id,
                    name: m.name.unwrap_or_default(),
                    contact_info: m.contact_info.unwrap_or_default(),
                    address: m.address.unwrap_or_default(),
                })
                .collect();
            Ok(Response::new(SuppliersResponse { items }))
        }
        Err(e) => Err(map_db_error_to_status(e)),
    }
}
