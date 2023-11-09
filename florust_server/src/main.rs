mod circular_vec;
mod data_source;
mod manager_and_data;
#[cfg(any(feature = "iinteger_default_plugin", feature = "uinteger_default_plugin", feature = "float_default_plugin"))]
mod default_plugins;

use manager_and_data::{ManagerAndData, ManagerAndDataError, DataType, IIntegerManagerAndData, UIntegerManagerAndData, FloatManagerAndData};
use rocket::{launch, routes};
use std::{collections::HashMap, sync::Arc};

use florust_common::FlorustServerPluginError;

#[cfg(feature = "iinteger_default_plugin")]
use default_plugins::DefaultIIntegerDataManager;

#[cfg(feature = "uinteger_default_plugin")]
use default_plugins::DefaultUIntegerDataManager;

#[cfg(feature = "float_default_plugin")]
use default_plugins::DefaultFloatDataManager;

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
    let mut managers = HashMap::new();

    #[cfg(feature = "iinteger_default_plugin")] {
        let iinteger_manager = Box::new(IIntegerManagerAndData::new(
            Box::new(DefaultIIntegerDataManager{}) as _,
            10
        )) as BoxedManagerAndData;
        managers.insert(iinteger_manager.manager_id(), iinteger_manager);
    }

    #[cfg(feature = "uinteger_default_plugin")] {
        let uinteger_manager = Box::new(UIntegerManagerAndData::new(
            Box::new(DefaultUIntegerDataManager{}) as _,
            10
        ));
        managers.insert(uinteger_manager.manager_id(), uinteger_manager);
    }

    #[cfg(feature = "float_default_plugin")] {
        let float_manager = Box::new(FloatManagerAndData::new(
            Box::new(DefaultFloatDataManager{}) as _,
            10
        ));
        managers.insert(float_manager.manager_id(), float_manager);
    }

    let florust_state = FlorustState {
        managers_and_data: Arc::new(managers),
    };

    rocket::build().manage(florust_state).mount(
        "/data_source",
        routes![
            data_source::register,
            data_source::unregister,
            data_source::json_upload_data,
            data_source::form_upload_data,
            data_source::get_data
        ],
    )
}
