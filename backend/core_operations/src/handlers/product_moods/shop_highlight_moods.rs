//! Moods to show on storefront "Shop by mood": walk newest products first, collect distinct mood ids.

use std::collections::{HashMap, HashSet};

use crate::handlers::db_errors::map_db_error_to_status;
use core_db_entities::entity::{product_mood_mapping, product_moods, products};
use proto::proto::core::{
    ShopHighlightMoodItem, ShopHighlightMoodsRequest, ShopHighlightMoodsResponse,
};
use sea_orm::{
    ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use tonic::{Request, Response, Status};

pub async fn shop_highlight_moods(
    txn: &DatabaseTransaction,
    request: Request<ShopHighlightMoodsRequest>,
) -> Result<Response<ShopHighlightMoodsResponse>, Status> {
    let req = request.into_inner();
    let recent_limit = req.recent_product_limit.unwrap_or(100).clamp(1, 500) as u64;
    let max_moods = req.max_moods.unwrap_or(4).clamp(1, 20) as usize;

    // Newest products first (highest ProductID = most recently created when IDs auto-increment).
    let product_rows = products::Entity::find()
        .order_by_desc(products::Column::ProductId)
        .limit(recent_limit)
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let product_ids: Vec<i64> = product_rows.iter().map(|p| p.product_id).collect();
    if product_ids.is_empty() {
        return Ok(Response::new(ShopHighlightMoodsResponse { items: vec![] }));
    }

    let mappings = product_mood_mapping::Entity::find()
        .filter(product_mood_mapping::Column::ProductId.is_in(product_ids.clone()))
        .all(txn)
        .await
        .map_err(map_db_error_to_status)?;

    let mut by_product: HashMap<i64, Vec<i64>> = HashMap::new();
    for m in mappings {
        by_product.entry(m.product_id).or_default().push(m.mood_id);
    }
    for mids in by_product.values_mut() {
        mids.sort_unstable();
        mids.dedup();
    }

    let mut seen: HashSet<i64> = HashSet::new();
    let mut ordered_mood_ids: Vec<i64> = Vec::new();

    'outer: for pid in &product_ids {
        if let Some(moods) = by_product.get(pid) {
            for &mid in moods {
                if seen.insert(mid) {
                    ordered_mood_ids.push(mid);
                    if ordered_mood_ids.len() >= max_moods {
                        break 'outer;
                    }
                }
            }
        }
    }

    let items: Vec<ShopHighlightMoodItem> = if ordered_mood_ids.is_empty() {
        vec![]
    } else {
        let mood_models = product_moods::Entity::find()
            .filter(product_moods::Column::MoodId.is_in(ordered_mood_ids.clone()))
            .all(txn)
            .await
            .map_err(map_db_error_to_status)?;

        let name_by_id: HashMap<i64, String> = mood_models
            .into_iter()
            .map(|m| (m.mood_id, m.mood_name))
            .collect();

        ordered_mood_ids
            .into_iter()
            .filter_map(|id| {
                name_by_id.get(&id).map(|name| ShopHighlightMoodItem {
                    mood_id: id,
                    mood_name: name.clone(),
                })
            })
            .collect()
    };

    // Newest products may have no mood links while older products do; still surface catalog moods.
    if items.is_empty() {
        let mood_models = product_moods::Entity::find()
            .order_by_desc(product_moods::Column::MoodId)
            .limit(max_moods as u64)
            .all(txn)
            .await
            .map_err(map_db_error_to_status)?;
        let fallback: Vec<ShopHighlightMoodItem> = mood_models
            .into_iter()
            .map(|m| ShopHighlightMoodItem {
                mood_id: m.mood_id,
                mood_name: m.mood_name,
            })
            .collect();
        return Ok(Response::new(ShopHighlightMoodsResponse {
            items: fallback,
        }));
    }

    Ok(Response::new(ShopHighlightMoodsResponse { items }))
}
