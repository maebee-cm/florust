use florust_common::{UploadedData, FlorustServerPluginError};
use rocket::{form::Form, post, put, Responder, State};

use crate::{FlorustState, manager_and_data::ManagerAndDataError};

#[derive(Responder)]
pub enum DataSourceError {
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 409, content_type = "json")]
    Conflict(String),
    #[response(status = 500, content_type = "json")]
    InternalError(String)
}

impl From<ManagerAndDataError> for DataSourceError {
    fn from(value: ManagerAndDataError) -> Self {
        match &value {
            ManagerAndDataError::DataSourceManager(error) => match error {
                FlorustServerPluginError::DataSourceAlreadyExists(_) | FlorustServerPluginError::DataSourceAlreadyDeregistered(_) => Self::Conflict(
                    serde_json::to_string(&value)
                        .expect("Failed to serialize valid ManagerAndDataError")
                ),
                FlorustServerPluginError::DataSourceDoesntExist(_) | FlorustServerPluginError::DataSourceManagerDoesntExist(_)=> Self::NotFound(
                    serde_json::to_string(&value)
                        .expect("Failed to serialize valid ManagerAndDataError")
                ),
                FlorustServerPluginError::DataSourceManager(_) => Self::BadRequest(
                    serde_json::to_string(&value)
                        .expect("Failed to serialize valid ManagerAndDataError")
                ),
            },
            ManagerAndDataError::NoData => Self::InternalError(
                serde_json::to_string(&value)
                    .expect("Failed to serialize valid ManagerAndDataError")
            ),
            ManagerAndDataError::IndexOutOfBounds => Self::InternalError(
                serde_json::to_string(&value)
                    .expect("Failed to serialize valid ManagerAndDataError")
            ),
        }
    }
}

#[derive(Responder)]
#[response(status = 200)]
pub struct OkResponder<T>(T) where T: Send + Sync;

#[post("/register/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn register(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Option<Form<UploadedData>>
) -> Result<OkResponder<()>, DataSourceError> {
    if let Some(data) = data {
        state.register_data_source(&manager_id, data_source_id, Some(data.data.as_slice())).await
            .map(OkResponder)
            .map_err(DataSourceError::from)
    }
    else {
        state.register_data_source(&manager_id, data_source_id, None).await
            .map(OkResponder)
            .map_err(DataSourceError::from)
    }
}

#[post("/unregister/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn unregister(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Option<Form<UploadedData>>
) -> Result<OkResponder<()>, DataSourceError> {
    if let Some(data) = data {
        state.deregister_data_source(&manager_id, &data_source_id, Some(data.data.as_slice())).await
            .map(OkResponder)
            .map_err(DataSourceError::from)
    }
    else {
        state.deregister_data_source(&manager_id, &data_source_id, None).await
            .map(OkResponder)
            .map_err(DataSourceError::from)
    }
}

#[put("/upload_data/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn upload_data(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Form<UploadedData>,
) -> Result<OkResponder<()>, DataSourceError> {
    state.update_data(&manager_id, &data_source_id, data.data.as_slice()).await
        .map(OkResponder)
        .map_err(DataSourceError::from)
}
