mod data_source;

use std::{collections::HashMap, sync::Arc};
use rocket::{launch, routes};

use florust_common::server_plugin::DataSourceManager;

pub struct FlorustState {
    managers: Arc<HashMap<&'static str, Box<dyn DataSourceManager>>>
}

impl FlorustState {
    pub fn manager_exists(&self, manager_id: &str) -> bool {
        self.managers.contains_key(manager_id)
    }

    pub fn get_manager(&self, manager_id: &str) -> Option<&Box<dyn DataSourceManager>> {
        self.managers.get(manager_id)
    }
}

#[launch]
fn launch() -> _ {
    let florust_state = FlorustState {
        managers: Arc::new(HashMap::new())
    };

    rocket::build()
        .manage(florust_state)
        .mount("/data_source", routes![
            data_source::register,
            data_source::unregister,
            data_source::upload_data
        ])
}