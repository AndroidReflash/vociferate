//! Database UniFFI exports

use crate::database::database::{DeviceDatabase, DbError};
use crate::uniffi_exports::{VoiceError, DEVICE_DB, SETTING_KEY_DEVICE_NAME, get_db};
use std::sync::Arc;

#[uniffi::export]
pub fn init_database(path: String) -> Result<(), VoiceError> {
    let db = DeviceDatabase::open(path).map_err(|e| VoiceError::Other(e.to_string()))?;

    let device_id = match db.get_device_id().map_err(|e| VoiceError::Other(e.to_string()))? {
        Some(id) => id,
        None => {
            let new_id = uuid::Uuid::new_v4().to_string();
            db.set_device_id(new_id.clone()).map_err(|e| VoiceError::Other(e.to_string()))?;
            new_id
        }
    };

    crate::network::config::init_device_id(Some(device_id));

    let mut guard = DEVICE_DB.lock().unwrap();
    *guard = Some(Arc::new(db));
    Ok(())
}

#[uniffi::export]
pub fn get_device_id() -> Result<String, VoiceError> {
    Ok(crate::network::config::get_cached_device_id().to_string())
}

#[uniffi::export]
pub fn get_device_name() -> Result<Option<String>, VoiceError> {
    let db = get_db()?;
    db.get(SETTING_KEY_DEVICE_NAME.to_string())
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn set_device_name(name: String) -> Result<(), VoiceError> {
    let db = get_db()?;
    db.set(SETTING_KEY_DEVICE_NAME.to_string(), name)
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn delete_device_name() -> Result<(), VoiceError> {
    let db = get_db()?;
    db.delete(SETTING_KEY_DEVICE_NAME.to_string())
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn overwrite_device_name(name: String) -> Result<(), VoiceError> {
    let db = get_db()?;
    db.delete(SETTING_KEY_DEVICE_NAME.to_string()).ok();
    db.set(SETTING_KEY_DEVICE_NAME.to_string(), name)
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn get_bool_setting(key: String) -> Result<Option<bool>, VoiceError> {
    let db = get_db()?;
    db.get_bool(key)
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn set_bool_setting(key: String, value: bool) -> Result<(), VoiceError> {
    let db = get_db()?;
    db.set_bool(key, value)
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn set_string_setting(key: String, value: String) -> Result<(), VoiceError> {
    let db = get_db()?;
    db.set(key, value)
        .map_err(|e| VoiceError::Other(e.to_string()))
}

#[uniffi::export]
pub fn get_string_setting(key: String) -> Result<Option<String>, VoiceError> {
    let db = get_db()?;
    db.get(key)
        .map_err(|e| VoiceError::Other(e.to_string()))
}