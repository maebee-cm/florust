use async_trait::async_trait;

use crate::server_data_source_error::DataSourceError;

#[async_trait]
pub trait DataSourceManager: Sync + Send {
    fn data_source_type(&self) -> &'static str;

    fn data_type_length(&self) -> usize;

    async fn register(&self, id: String) -> Result<(), DataSourceError>;

    async fn unregister(&self, id: &str) -> Result<(), DataSourceError>;

    async fn update_data(&self, id: &str, data: &[u8]) -> Result<(), DataSourceError>;

    async fn get_data(&self, id: &str, out: &mut [u8]) -> Result<(), DataSourceError>;
}

pub type CreateAtomicDataSourceManager = unsafe extern fn() -> Box<Box<dyn DataSourceManager>>;