use rocket::async_trait;

use crate::server_data_source_error::DataSourceManagerError;

#[async_trait]
pub trait DataSourceManager: Sync + Send {
    fn manager_id(&self) -> &'static str;

    fn register(&self, id: String, data: Option<&[u8]>) -> Result<(), DataSourceManagerError>;

    fn unregister(&self, id: &str, data: Option<&[u8]>) -> Result<(), DataSourceManagerError>;
}

#[async_trait]
pub trait IIntegerDataSourceManager: DataSourceManager + Sync + Send {
    async fn update_data(&self, id: &str, data: &[u8]) -> Result<i64, DataSourceManagerError>;
}

#[async_trait]
pub trait UIntegerDataSourceManager: DataSourceManager + Sync + Send {
    async fn update_data(&self, id: &str, data: &[u8]) -> Result<u64, DataSourceManagerError>;
}

#[async_trait]
pub trait FloatDataSourceManager: DataSourceManager + Sync + Send {
    async fn update_data(&self, id: &str, data: &[u8]) -> Result<f64, DataSourceManagerError>;
}

pub type FFIBoxTrait<T> = Box<Box<T>>;

pub type CreateIIntegerDataSourceManager = unsafe extern "C" fn() -> Option<FFIBoxTrait<dyn IIntegerDataSourceManager>>;
pub type CreateUIntegerDataSourceManager = unsafe extern "C" fn() -> Option<FFIBoxTrait<dyn UIntegerDataSourceManager>>;
pub type CreateFloatDataSourceManager = unsafe extern "C" fn() -> Option<FFIBoxTrait<dyn FloatDataSourceManager>>;