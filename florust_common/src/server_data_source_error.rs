use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Error, Debug)]
pub enum DataSourceManagerError {
    #[error(
        "Attempted to register data source ID, but ID already exists in given data source manager."
    )]
    IdAlreadyExists,
    #[error(
        "Attempted to access data source ID, but ID doesn't exist in given data source manager."
    )]
    IdDoesntExist,
    #[error("Attempted to interact with data source manager, but manager doesn't exist")]
    DataSourceManagerDoesntExist,
    #[error("Failed to parse data provided to data source: {0}")]
    DataSourceParseFailure(String)
}
