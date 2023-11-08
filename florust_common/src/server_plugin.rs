use std::result;

use rocket::async_trait;
use serde::{Serialize, Deserialize};
use thiserror::Error;

//use crate::server_data_source_error::DataSourceManagerError;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DataSourceManagerError {
    #[error("DataSourceManager was given invalid data: {0}")]
    InvalidData(String)
}

/// A specialized [`Result`](result::Result) type for [`DataSourceManager`] operations.
/// 
/// This type was made to avoid having to write [`DataSourceManagerError`] repeatedly for return types
/// as they are used widely and repeatedly in both this module (`server_plugin`) and in the `florust_server`
/// crate.
pub type Result<T> = result::Result<T, DataSourceManagerError>;

/// A trait defining a base data source manager. This is a base type that is used the specialized
/// managers below. This type simply serves as a template to define the functionality that all specialized
/// data manager types share.
#[async_trait]
pub trait DataSourceManager<T>: Sync + Send {
    /// Returns the id associated with the data manager.
    fn manager_id(&self) -> &'static str;

    /// Called when a new data source registers itself to the id belonging to the data source manager.
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn register(&self, id: String) -> Result<()>;

    /// Called when a new data source registers itself to the id belonging to the data source manager.
    /// This method is chosen if the data source provided additional info with the registration request.
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn register_with_data(&self, id: String, data: &[u8]) -> Result<()>;

    /// Called when a data source requests to be deregistered from the data source manager.
    /// 
    /// Returns the unit type if no errors occurred or a [`DataSourceManagerError`] in case of an error.
    async fn deregister(&self, id: &str) -> Result<()>;

    /// Called when a data source requests to be deregistered from the data source manager. This method
    /// is chosen if the data source provided additional info with the deregistration request.
    /// 
    /// Returns the unit type if no errors occurred, or a [`DataSourceManagerError`] in case of an error.
    async fn deregister_with_data(&self, id: &str, data: &[u8]) -> Result<()>;

    /// Called when a data source has posted an update. provides the raw data that the data source
    /// has sent to the Florust server.
    /// 
    /// Returns the value parsed from the data, or a [`DataSourceManagerError`] in case of an error.
    async fn update_data(&self, id: &str, data: &[u8]) -> Result<T>;
}

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`i64`] from data provided by a data source.
pub trait IIntegerDataSourceManager: DataSourceManager<i64> + Sync + Send {
}

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`u64`] from data provided by a data source.
pub trait UIntegerDataSourceManager: DataSourceManager<u64> + Sync + Send {
}

/// One of three specialized types of [`DataSourceManager`] that is responsible for producing data of
/// type [`f64`] from data provided by a data source.
pub trait FloatDataSourceManager: DataSourceManager<f64> + Sync + Send {
}

/// A type representing a double boxed trait. This type is double boxed as a boxed trait object is a fat
/// pointer which would be difficult to transport across FFI boundaries. Boxing the box resolves this issue
/// by making it a normal sized pointer.
pub type FFIBoxTrait<T> = Box<Box<T>>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`IIntegerDataSourceManager`].
pub type CreateIIntegerDataSourceManager = unsafe extern "C" fn() -> FFIBoxTrait<dyn IIntegerDataSourceManager>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`UIntegerDataSourceManager`].
pub type CreateUIntegerDataSourceManager = unsafe extern "C" fn() -> FFIBoxTrait<dyn UIntegerDataSourceManager>;

/// A function that returns a [`FFIBoxTrait`] which contains an [`FloatDataSourceManager`].
pub type CreateFloatDataSourceManager = unsafe extern "C" fn() -> FFIBoxTrait<dyn FloatDataSourceManager>;