//! Generic admin-configurable key/value settings store, plus typed accessors for the settings
//! groups that actually exist so far (currently just the abandoned-cart worker's schedule).
//! DB-backed so changes take effect on the worker's next tick with no restart — env vars
//! remain the fallback default for a key that's never been touched from the admin panel.

use crate::handlers::db_errors::map_db_error_to_status;
use crate::procedures::abandoned_cart::DEFAULT_ABANDONED_DELAY_HOURS;
use chrono::Utc;
use core_db_entities::entity::app_settings;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, EntityTrait};
use tonic::Status;

const ABANDONED_CART_ENABLED_KEY: &str = "abandoned_cart.enabled";
const ABANDONED_CART_DELAY_HOURS_KEY: &str = "abandoned_cart.delay_hours";
const ABANDONED_CART_POLL_INTERVAL_SEC_KEY: &str = "abandoned_cart.poll_interval_sec";

const DEFAULT_POLL_INTERVAL_SEC: i64 = 3600;
/// Floor to stop an admin from accidentally hammering the DB/outbox every few seconds.
const MIN_POLL_INTERVAL_SEC: i64 = 60;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbandonedCartSettings {
    pub enabled: bool,
    pub delay_hours: i64,
    pub poll_interval_sec: i64,
}

async fn get_setting<C: ConnectionTrait>(db: &C, key: &str) -> Result<Option<String>, Status> {
    Ok(app_settings::Entity::find_by_id(key)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?
        .map(|row| row.setting_value))
}

async fn set_setting<C: ConnectionTrait>(db: &C, key: &str, value: String) -> Result<(), Status> {
    let existing = app_settings::Entity::find_by_id(key)
        .one(db)
        .await
        .map_err(map_db_error_to_status)?;
    match existing {
        Some(row) => {
            let mut active: app_settings::ActiveModel = row.into();
            active.setting_value = ActiveValue::Set(value);
            active.updated_at = ActiveValue::Set(Utc::now());
            active.update(db).await.map_err(map_db_error_to_status)?;
        }
        None => {
            let active = app_settings::ActiveModel {
                setting_key: ActiveValue::Set(key.to_string()),
                setting_value: ActiveValue::Set(value),
                updated_at: ActiveValue::Set(Utc::now()),
            };
            active.insert(db).await.map_err(map_db_error_to_status)?;
        }
    }
    Ok(())
}

fn env_default_poll_interval_sec() -> i64 {
    std::env::var("ABANDONED_CART_POLL_INTERVAL_SEC")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_POLL_INTERVAL_SEC)
}

fn env_default_delay_hours() -> i64 {
    std::env::var("ABANDONED_CART_DELAY_HOURS")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(DEFAULT_ABANDONED_DELAY_HOURS)
}

fn env_default_enabled() -> bool {
    !std::env::var("ABANDONED_CART_DISABLE_WORKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Reads the abandoned-cart worker's live settings — a DB row (set via the admin panel)
/// wins over the env-var default, which wins over the hardcoded default. Called fresh on
/// every worker tick, so an admin's change takes effect within one poll interval, no restart.
pub async fn get_abandoned_cart_settings<C: ConnectionTrait>(
    db: &C,
) -> Result<AbandonedCartSettings, Status> {
    let enabled = match get_setting(db, ABANDONED_CART_ENABLED_KEY).await? {
        Some(v) => v == "1",
        None => env_default_enabled(),
    };
    let delay_hours = match get_setting(db, ABANDONED_CART_DELAY_HOURS_KEY).await? {
        Some(v) => v.parse::<i64>().unwrap_or_else(|_| env_default_delay_hours()),
        None => env_default_delay_hours(),
    };
    let poll_interval_sec = match get_setting(db, ABANDONED_CART_POLL_INTERVAL_SEC_KEY).await? {
        Some(v) => v
            .parse::<i64>()
            .unwrap_or_else(|_| env_default_poll_interval_sec()),
        None => env_default_poll_interval_sec(),
    };

    Ok(AbandonedCartSettings {
        enabled,
        delay_hours: delay_hours.max(1),
        poll_interval_sec: poll_interval_sec.max(MIN_POLL_INTERVAL_SEC),
    })
}

/// Updates only the fields provided, then returns the resulting full settings — same
/// "partial update, full echo back" shape used elsewhere in this codebase (e.g. shipping
/// addresses). `poll_interval_sec` is floored to `MIN_POLL_INTERVAL_SEC` to stop an admin from
/// accidentally configuring a near-continuous sweep.
pub async fn update_abandoned_cart_settings<C: ConnectionTrait>(
    db: &C,
    enabled: Option<bool>,
    delay_hours: Option<i64>,
    poll_interval_sec: Option<i64>,
) -> Result<AbandonedCartSettings, Status> {
    if let Some(v) = enabled {
        set_setting(
            db,
            ABANDONED_CART_ENABLED_KEY,
            if v { "1".to_string() } else { "0".to_string() },
        )
        .await?;
    }
    if let Some(v) = delay_hours {
        if v <= 0 {
            return Err(Status::invalid_argument("delay_hours must be positive"));
        }
        set_setting(db, ABANDONED_CART_DELAY_HOURS_KEY, v.to_string()).await?;
    }
    if let Some(v) = poll_interval_sec {
        if v <= 0 {
            return Err(Status::invalid_argument("poll_interval_sec must be positive"));
        }
        set_setting(db, ABANDONED_CART_POLL_INTERVAL_SEC_KEY, v.to_string()).await?;
    }

    get_abandoned_cart_settings(db).await
}
