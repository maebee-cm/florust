use std::array::TryFromSliceError;

use florust_common::server::{DataSourceManager, self, DataSourceManagerError};
use rocket::async_trait;

pub struct DefaultIIntegerDataManager;

#[async_trait]
impl DataSourceManager<i64> for DefaultIIntegerDataManager {
    fn manager_id(&self) ->  &'static str {
        "FlorustDefaultIIntegerDataManager"
    }

    async fn register(&self, _id: String) -> server::Result<()> {
        Ok(())
    }

    async fn register_with_data(&self, _id: String, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn deregister(&self, _id: &str) -> server::Result<()> {
        Ok(())
    }

    async fn deregister_with_data(&self, _id: &str, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn update_data(&self, _id: &str, data: &[u8]) -> server::Result<i64> {
        let data = data.try_into()
            .map_err(|err:  TryFromSliceError| DataSourceManagerError::InvalidData(err.to_string()))?;
        Ok(i64::from_be_bytes(data))
    }
}

pub struct DefaultUIntegerDataManager;

#[async_trait]
impl DataSourceManager<u64> for DefaultUIntegerDataManager {
    fn manager_id(&self) ->  &'static str {
        "FlorustDefaultUIntegerDataManager"
    }

    async fn register(&self, _id: String) -> server::Result<()> {
        Ok(())
    }

    async fn register_with_data(&self, _id: String, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn deregister(&self, _id: &str) -> server::Result<()> {
        Ok(())
    }

    async fn deregister_with_data(&self, _id: &str, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn update_data(&self, _id: &str, data: &[u8]) -> server::Result<u64> {
        let data = data.try_into()
            .map_err(|err:  TryFromSliceError| DataSourceManagerError::InvalidData(err.to_string()))?;
        Ok(u64::from_be_bytes(data))
    }
}

pub struct DefaultFloatDataManager;

#[async_trait]
impl DataSourceManager<f64> for DefaultFloatDataManager {
    fn manager_id(&self) ->  &'static str {
        "FlorustDefaultFloatDataManager"
    }

    async fn register(&self, _id: String) -> server::Result<()> {
        Ok(())
    }

    async fn register_with_data(&self, _id: String, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn deregister(&self, _id: &str) -> server::Result<()> {
        Ok(())
    }

    async fn deregister_with_data(&self, _id: &str, _data: &[u8]) -> server::Result<()> {
        Ok(())
    }

    async fn update_data(&self, _id: &str, data: &[u8]) -> server::Result<f64> {
        let data = data.try_into()
            .map_err(|err:  TryFromSliceError| DataSourceManagerError::InvalidData(err.to_string()))?;
        Ok(f64::from_be_bytes(data))
    }
}
