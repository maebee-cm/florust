use async_trait::async_trait;

use crate::server_data_source_error::DataSourceError;

#[async_trait]
pub trait AtomicDataSourceManager: Sync + Send {
    fn data_source_type(&self) -> &'static str;

    async fn register(&mut self, id: String) -> Result<(), DataSourceError>;

    async fn unregister(&mut self, id: &str) -> Result<(), DataSourceError>;

    async fn update_data(&self, id: &str) -> Result<(), DataSourceError>;
}

#[async_trait]
pub trait DataSourceManager: Sync + Send {
    fn data_source_type(&self) -> &'static str;

    async fn register(&mut self, id: String) -> Result<(), DataSourceError>;

    async fn unregister(&mut self, id: &str) -> Result<(), DataSourceError>;

    async fn update_data(&mut self, id: &str) -> Result<(), DataSourceError>;
}

pub type CreateAtomicDataSourceManager = unsafe extern fn() -> Box<Box<dyn AtomicDataSourceManager>>;
pub type CreateDataSourceManager = unsafe extern fn() -> Box<Box<dyn DataSourceManager>>;