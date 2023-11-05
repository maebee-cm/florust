mod circular_vec;
mod data_source;
mod manager_and_data;

use rocket::{launch, routes};
use std::{collections::HashMap, sync::Arc};

use florust_common::{server_data_source_error::DataSourceManagerError, server_plugin};

type ManagerAndData = Box<dyn manager_and_data::ManagerAndData>;

pub struct FlorustState {
    managers_and_data: Arc<
        HashMap<
            &'static str,
            ManagerAndData,
        >,
    >,
}

impl FlorustState {
    pub fn manager_exists(&self, manager_id: &str) -> bool {
        self.managers_and_data.contains_key(manager_id)
    }

    pub fn get_manager(&self, manager_id: &str) -> Option<&ManagerAndData> {
        self.managers_and_data
            .get(manager_id)
            .and_then(|v| Some(v))
    }

    pub async fn register_data_source(&self, manager_id: &str, data_source_id: String, data: Option<&[u8]>) -> server_plugin::Result<()> {
        if let Some(data) = data {
            self.managers_and_data
                .get(manager_id)
                .ok_or(DataSourceManagerError::DataSourceManagerDoesntExist)?
                .register_with_data(data_source_id, data).await
        }
        else {
            self.managers_and_data
                .get(manager_id)
                .ok_or(DataSourceManagerError::DataSourceManagerDoesntExist)?
                .register(data_source_id).await
        }
    }

    pub async fn deregister_data_source(&self, manager_id: &str, data_source_id: &str, data: Option<&[u8]>) -> server_plugin::Result<()> {
        if let Some(data) = data {
            self.managers_and_data
                .get(manager_id)
                .ok_or(DataSourceManagerError::DataSourceManagerDoesntExist)?
                .deregister_with_data(data_source_id, data).await
        }
        else {
            self.managers_and_data
                .get(manager_id)
                .ok_or(DataSourceManagerError::DataSourceManagerDoesntExist)?
                .deregister(data_source_id).await
        }
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
