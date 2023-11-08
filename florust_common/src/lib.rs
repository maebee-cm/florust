use rocket::FromForm;

pub mod server_plugin;

use serde::{Serialize, Deserialize};
use server_plugin::DataSourceManagerError;
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum FlorustServerPluginError {
    #[error("Attempted to register data source ID ({0}), but it already exists.")]
    DataSourceAlreadyExists(String),
    #[error("Attempted to access data source ID ({0}), but ID doesn't exist.")]
    DataSourceDoesntExist(String),
    #[error("Attempted to deregister data source ID ({0}), but it already was deregistered")]
    DataSourceAlreadyDeregistered(String),
    #[error("Attempted to access data source manager ({0}), but manager doesn't exist")]
    DataSourceManagerDoesntExist(String),
    #[error("Data source manager failed with error: {0}")]
    DataSourceManager(DataSourceManagerError),
}


#[derive(FromForm, Serialize, Deserialize)]
pub struct UploadedData {
    pub data: Vec<u8>
}
