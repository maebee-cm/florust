mod circular_vec;
mod data_source;
mod manager_and_data;

use manager_and_data::ManagerAndDataError;
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

    pub fn get_manager(&self, manager_id: &str) -> Option<&BoxedManagerAndData> {
        self.managers_and_data
            .get(manager_id)
            .and_then(|v| Some(v))
    }

    pub async fn register_data_source(&self, manager_id: &str, data_source_id: String, data: Option<&[u8]>) -> manager_and_data::Result<()> {
        if let Some(data) = data {
            self.managers_and_data
                .get(manager_id)
                .ok_or(
                    ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                        )
                    )
                )?
                .register_with_data(data_source_id, data).await
        }
        else {
            self.managers_and_data
                .get(manager_id)
                .ok_or(
                    ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                        )
                    )
                )?
                .register(data_source_id).await
        }
    }

    pub async fn deregister_data_source(&self, manager_id: &str, data_source_id: &str, data: Option<&[u8]>) -> manager_and_data::Result<()> {
        if let Some(data) = data {
            self.managers_and_data
                .get(manager_id)
                .ok_or(
                    ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                        )
                    )
                )?
                .deregister_with_data(data_source_id, data).await
        }
        else {
            self.managers_and_data
                .get(manager_id)
                .ok_or(
                    ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(manager_id.to_string()
                        )
                    )
                )?
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
