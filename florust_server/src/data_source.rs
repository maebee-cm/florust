use florust_common::{
    server_data_source_error::DataSourceManagerError,
    UploadedData
};
use rocket::{form::Form, post, put, Responder, State};

use crate::FlorustState;

#[derive(Responder)]
pub enum DataSourceError {
    #[response(status = 400, content_type = "json")]
    BadRequest(String),
    #[response(status = 404, content_type = "json")]
    NotFound(String),
    #[response(status = 409, content_type = "json")]
    Conflict(String)
}

impl From<DataSourceManagerError> for DataSourceError {
    fn from(value: DataSourceManagerError) -> Self {
        match &value {
            DataSourceManagerError::DataSourceParseFailure(_) => Self::BadRequest(
                serde_json::to_string(&value)
                    .expect("Failed to serialize valid DataSourceManagerError")
            ),
            DataSourceManagerError::IdAlreadyExists => Self::Conflict(
                serde_json::to_string(&value)
                    .expect("Failed to serialize valid DataSourceManagerError")
            ),
            DataSourceManagerError::IdDoesntExist | DataSourceManagerError::DataSourceManagerDoesntExist => {
                Self::NotFound(serde_json::to_string(&value)
                    .expect("Failed to serialize valid DataSourceManagerError")
                )
            },
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
