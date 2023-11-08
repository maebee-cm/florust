mod circular_vec;
mod data_source;
mod manager_and_data;

use manager_and_data::{ManagerAndDataError, DataType};
use rocket::{launch, routes};
use std::{collections::HashMap, sync::Arc};

use florust_common::FlorustServerPluginError;

type BoxedManagerAndData = Box<dyn manager_and_data::ManagerAndData>;

pub struct FlorustState {
    managers_and_data: Arc<
        HashMap<
            &'static str,
            BoxedManagerAndData,
        >,
    >,
}

impl FlorustState {
    pub fn manager_exists(&self, manager_id: &str) -> bool {
        self.managers_and_data.contains_key(manager_id)
    }

    pub fn get_manager_or_err(&self, manager_id: &str) -> manager_and_data::Result<&BoxedManagerAndData> {
        self.managers_and_data
            .get(manager_id)
            .ok_or(
                ManagerAndDataError::DataSourceManager(
                        FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                    )
                )
            )
    }

    pub async fn register_data_source(&self, manager_id: &str, data_source_id: String, data: Option<&[u8]>) -> manager_and_data::Result<()> {
        if let Some(data) = data {
            self.get_manager_or_err(manager_id)?
                .register_with_data(data_source_id, data).await
        }
        else {
            self.get_manager_or_err(manager_id)?
                .register(data_source_id).await
        }
    }

    pub async fn deregister_data_source(&self, manager_id: &str, data_source_id: &str, data: Option<&[u8]>) -> manager_and_data::Result<()> {
        if let Some(data) = data {
            self.get_manager_or_err(manager_id)?
                .deregister_with_data(data_source_id, data).await
        }
        else {
            self.get_manager_or_err(manager_id)?
                .deregister(data_source_id).await
        }
    }

    pub async fn update_data(&self, manager_id: &str, data_source_id: &str, data: &[u8]) -> manager_and_data::Result<()> {
        self.managers_and_data
            .get(manager_id)
            .ok_or(
                ManagerAndDataError::DataSourceManager(
                        FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                    )
                )
            )?
            .update_data(data_source_id, data).await
    }

    pub async fn get_data(&self, manager_id: &str, data_source_id: &str, index: usize) -> manager_and_data::Result<DataType> {
        self.get_manager_or_err(manager_id)?
            .get_data(data_source_id, index).await
    }
}

#[launch]
fn launch() -> _ {
    let florust_state = FlorustState {
        managers_and_data: Arc::new(HashMap::new()),
    };

    rocket::build().manage(florust_state).mount(
        "/data_source",
        routes![
            data_source::register,
            data_source::unregister,
            data_source::upload_data
        ],
    )
}
