use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DataSourceError {
    #[error("Attempted to register data source ID, but ID already exists in given data source type manager.")]
    IdAlreadyExists,
    #[error("Attempted to access data source ID, but ID doesn't exist in given data source type manager.")]
    IdDoesntExist,
    #[error("Attempted to interact with data source type, but manager doesn't exist")]
    DataSourceTypeDoesntExist
}