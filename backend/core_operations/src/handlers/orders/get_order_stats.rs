//! P1 Admin dashboard aggregation: order counts/revenue by status + customer count, computed with
//! SQL COUNT/SUM/GROUP BY instead of the caller paginating the entire Orders/Users tables and
//! aggregating client-side (see admin-queries.ts's fetchOrderCountsByStatus/fetchCustomerCount/
//! fetchDashboardStats/fetchDashboardExtras, which this RPC replaces).

use crate::handlers::db_errors::map_db_error_to_status;
use chrono::{DateTime, Utc};
use core_db_entities::entity::{orders, users};
use proto::proto::core::{GetOrderStatsRequest, GetOrderStatsResponse, OrderStatusCount};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, DbBackend, EntityTrait, PaginatorTrait,
    QueryFilter, Statement,
};
use tonic::{Request, Response, Status};

pub async fn get_order_stats(
    txn: &DatabaseTransaction,
    request: Request<GetOrderStatsRequest>,
) -> Result<Response<GetOrderStatsResponse>, Status> {
    let req = request.into_inner();

    let start_dt = req
        .order_date_start
        .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0));
    let end_dt = req
        .order_date_end
        .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0));

    // total_orders: real SQL COUNT(*), not fetch-all-then-len() — matches the .count(txn)
    // pattern already used elsewhere in this crate (e.g. cancellation_saga.rs, coupons/eligibility.rs).
    let mut count_query = orders::Entity::find();
    if let Some(dt) = start_dt {
        count_query = count_query.filter(orders::Column::OrderDate.gte(dt));
    }
    if let Some(dt) = end_dt {
        count_query = count_query.filter(orders::Column::OrderDate.lte(dt));
    }
    let total_orders = count_query
        .count(txn)
        .await
        .map_err(map_db_error_to_status)? as i64;

    let customer_count = users::Entity::find()
        .count(txn)
        .await
        .map_err(map_db_error_to_status)? as i64;

    // Revenue and per-status counts need SQL-level SUM/GROUP BY, which SeaORM's fluent query
    // builder doesn't offer a simple typed helper for — build the WHERE clause dynamically so an
    // absent date filter doesn't need a placeholder bound twice (IS NULL OR ... pattern), matching
    // this crate's existing raw-SQL convention (Statement::from_sql_and_values, e.g. cancel_order_items.rs).
    let mut where_clauses: Vec<String> = Vec::new();
    let mut binds: Vec<sea_orm::Value> = Vec::new();
    if let Some(dt) = start_dt {
        where_clauses.push("Orders.OrderDate >= ?".to_string());
        binds.push(dt.into());
    }
    if let Some(dt) = end_dt {
        where_clauses.push("Orders.OrderDate <= ?".to_string());
        binds.push(dt.into());
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let revenue_sql = format!(
        "SELECT CAST(COALESCE(SUM(Orders.grand_total_minor), 0) AS SIGNED) AS total_revenue_paise \
         FROM Orders {where_sql}"
    );
    let revenue_row = txn
        .query_one(Statement::from_sql_and_values(
            DbBackend::MySql,
            revenue_sql,
            binds.clone(),
        ))
        .await
        .map_err(map_db_error_to_status)?;
    let total_revenue_paise: i64 = revenue_row
        .and_then(|r| r.try_get::<i64>("", "total_revenue_paise").ok())
        .unwrap_or(0);

    let by_status_sql = format!(
        "SELECT Orders.StatusID AS status_id, OrderStatus.StatusName AS status_name, \
         CAST(COUNT(*) AS SIGNED) AS cnt \
         FROM Orders JOIN OrderStatus ON Orders.StatusID = OrderStatus.StatusID \
         {where_sql} GROUP BY Orders.StatusID, OrderStatus.StatusName"
    );
    let by_status_rows = txn
        .query_all(Statement::from_sql_and_values(
            DbBackend::MySql,
            by_status_sql,
            binds,
        ))
        .await
        .map_err(map_db_error_to_status)?;
    let by_status: Vec<OrderStatusCount> = by_status_rows
        .into_iter()
        .filter_map(|row| {
            let status_id = row.try_get::<i64>("", "status_id").ok()?;
            let status_name = row.try_get::<String>("", "status_name").ok()?;
            let count = row.try_get::<i64>("", "cnt").ok()?;
            Some(OrderStatusCount {
                status_id,
                status_name,
                count,
            })
        })
        .collect();

    Ok(Response::new(GetOrderStatsResponse {
        total_orders,
        total_revenue_paise,
        by_status,
        customer_count,
    }))
}
