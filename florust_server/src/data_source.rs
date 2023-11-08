use florust_common::{UploadedData, FlorustServerPluginError};
use rocket::{form::Form, post, put, get, Responder, State, serde::json::Json};

use crate::{FlorustState, manager_and_data::{ManagerAndDataError, DataType, self}};

#[derive(Responder)]
pub enum DataSourceError {
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<ManagerAndDataError>),
    #[response(status = 404, content_type = "json")]
    NotFound(Json<ManagerAndDataError>),
    #[response(status = 409, content_type = "json")]
    Conflict(Json<ManagerAndDataError>),
    #[response(status = 500, content_type = "json")]
    InternalError(Json<ManagerAndDataError>)
}

impl From<ManagerAndDataError> for DataSourceError {
    fn from(value: ManagerAndDataError) -> Self {
        match &value {
            ManagerAndDataError::DataSourceManager(error) => match error {
                FlorustServerPluginError::DataSourceAlreadyExists(_) | FlorustServerPluginError::DataSourceAlreadyDeregistered(_) => Self::Conflict(
                    Json(value)
                ),
                FlorustServerPluginError::DataSourceDoesntExist(_) | FlorustServerPluginError::DataSourceManagerDoesntExist(_)=> Self::NotFound(
                    Json(value)
                ),
                FlorustServerPluginError::DataSourceManager(_) => Self::BadRequest(
                    Json(value)
                ),
            },
            ManagerAndDataError::NoData => Self::InternalError(
                Json(value)
            ),
            ManagerAndDataError::IndexOutOfBounds => Self::InternalError(
                Json(value)
            ),
        }
    }
}

#[derive(Responder)]
#[response(status = 200)]
pub struct OkResponder<T>(Json<T>) where T: Send + Sync;

fn state_op_to_responder<T: Send + Sync>(op_result: manager_and_data::Result<T>) -> Result<OkResponder<T>, DataSourceError> {
    op_result.map(|value| OkResponder(Json(value)))
        .map_err(DataSourceError::from)
}

#[post("/register/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn register(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Option<Form<UploadedData>>
) -> Result<OkResponder<()>, DataSourceError> {
    let data = match &data {
        Some(data) => Some(data.data.as_slice()),
        None => None
    };

    state_op_to_responder(state.register_data_source(&manager_id, data_source_id, data).await)
}

#[post("/unregister/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn unregister(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Option<Form<UploadedData>>
) -> Result<OkResponder<()>, DataSourceError> {
    let data = match &data {
        Some(data) => Some(data.data.as_slice()),
        None => None
    };

    state_op_to_responder(state.deregister_data_source(&manager_id, &data_source_id, data).await)
}

#[put("/upload_data/<manager_id>/<data_source_id>", data = "<data>")]
pub async fn upload_data(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    data: Form<UploadedData>,
) -> Result<OkResponder<()>, DataSourceError> {

    state_op_to_responder(state.update_data(&manager_id, &data_source_id, data.data.as_slice()).await)
}

#[get("/<manager_id>/<data_source_id>/<index>")]
pub async fn get_data(
    state: &State<FlorustState>,
    manager_id: String,
    data_source_id: String,
    index: usize
) -> Result<OkResponder<DataType>, DataSourceError> {
    state_op_to_responder(state.get_data(&manager_id, &data_source_id, index).await)
}
