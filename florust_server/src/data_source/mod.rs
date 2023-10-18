use florust_common::{server_data_source_error::DataSourceManagerError, server_plugin::DataSourceManager, UploadedData};
use rocket::{post, put, State, Responder, form::Form};

use crate::FlorustState;

#[derive(Responder)]
#[response(status = 404, content_type = "json")]
pub struct NotFoundErrorResponder(String);

#[derive(Responder)]
#[response(status = 200)]
pub struct OkResponder(());

fn manager_exists_or_err<'a>(state: &'a State<FlorustState>, manager_id: &str) -> Result<&'a Box<dyn DataSourceManager>, NotFoundErrorResponder> {
    state.get_manager(manager_id).ok_or(NotFoundErrorResponder(
        serde_json::to_string(&DataSourceManagerError::DataSourceManagerDoesntExist)
            .expect("Failed to serialize valid DataSourceError::DataSourceTypeDoesntExist")
    ))
}

fn manager_op_success_or_err(manager_id: &str, result: Result<(), DataSourceManagerError>) -> Result<OkResponder, NotFoundErrorResponder>{
    result.map(OkResponder).map_err(|err| NotFoundErrorResponder(
        serde_json::to_string(&err)
            .expect(&format!("Data source manager (name: {}) returned unserializable error!", manager_id))
    ))
}

#[post("/register/<manager_id>/<id>")]
pub async fn register(state: &State<FlorustState>, manager_id: String, id: String) -> Result<OkResponder, NotFoundErrorResponder> {
    manager_op_success_or_err(&manager_id, manager_exists_or_err(state, &manager_id)?.register(id).await)
}

#[post("/unregister/<manager_id>/<id>")]
pub async fn unregister(state: &State<FlorustState>, manager_id: String, id: String) -> Result<OkResponder, NotFoundErrorResponder> {
    manager_op_success_or_err(&manager_id, manager_exists_or_err(state, &manager_id)?.unregister(&id).await)
}

#[put("/upload_data/<manager_id>/<id>", data="<data>")]
pub async fn upload_data(state: &State<FlorustState>, manager_id: String, id: String, data: Form<UploadedData>) -> Result<OkResponder, NotFoundErrorResponder> {
    manager_op_success_or_err(&manager_id, manager_exists_or_err(state, &manager_id)?.update_data(&id, &data.data).await)
}