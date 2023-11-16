mod circular_vec;
mod data_source;
mod manager_and_data;
#[cfg(any(feature = "iinteger_default_plugin", feature = "uinteger_default_plugin", feature = "float_default_plugin"))]
mod default_plugins;

use log::{info, warn};
use manager_and_data::{ManagerAndDataError, DataType, IIntegerManagerAndData, UIntegerManagerAndData, FloatManagerAndData};
use rocket::{launch, routes, serde::{Serialize, Deserialize}};
use toml::Table;
use std::{collections::HashMap, sync::Arc, fs::{read_dir, read_to_string}};

use florust_common::server::{FlorustServerPluginError, CreateIIntegerDataSourceManager, CreateUIntegerDataSourceManager, CreateFloatDataSourceManager};

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

fn default_max_data() -> usize { 10 }

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct FlorustServerPluginConfig {
    name: String,
    lib: String,
    #[serde(default = "default_max_data")]
    max_data: usize,
    data_type: String,
    create_func: Option<String>
}

impl FlorustServerPluginConfig {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lib(&self) -> &str {
        &self.lib
    }

    pub fn max_data(&self) -> usize {
        self.max_data
    }

    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    pub fn create_func(&self) -> Option<&str> {
        self.create_func.as_deref()
    }
}

#[launch]
fn launch() -> _ {
    let mut managers = HashMap::new();
    for plugin in load_plugins() {
        if let Some(_) = managers.get(plugin.manager_id()) {
            warn!("Skipping plugin (id: {}) because a plugin with the same id already exists", plugin.manager_id());
            continue;
        }

        managers.insert(plugin.manager_id(), plugin);
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

fn load_plugins() -> Vec<BoxedManagerAndData> {
    let mut plugins = Vec::new();

    // Load default plugins if they are enabled.
    #[cfg(feature = "iinteger_default_plugin")] {
        info!("Loading default plugin: FlorustDefaultIIntegerDataManager");

        let iinteger_manager = Box::new(IIntegerManagerAndData::new(
            Box::new(DefaultIIntegerDataManager{}) as _,
            10
        )) as BoxedManagerAndData;
        plugins.push(iinteger_manager);
    }

    #[cfg(feature = "uinteger_default_plugin")] {
        info!("Loading default plugin: FlorustDefaultUIntegerDataManager");

        let uinteger_manager = Box::new(UIntegerManagerAndData::new(
            Box::new(DefaultUIntegerDataManager{}) as _,
            10
        ));
        plugins.push(uinteger_manager);
    }

    #[cfg(feature = "float_default_plugin")] {
        info!("Loading default plugin: FlorustDefaultFloatDataManager");

        let float_manager = Box::new(FloatManagerAndData::new(
            Box::new(DefaultFloatDataManager{}) as _,
            10
        ));
        plugins.push(float_manager);
    }

    info!("Checking for custom plugins");
    let custom_plugin_dirs = match read_dir("plugins/") {
        Ok(entries) => entries,
        Err(_) => {
            info!("Plugins dir not found, not loading any plugins");
            return plugins;
        }
    };

    for plugin_dir in custom_plugin_dirs {
        // Only process entries which are dirs
        let plugin_dir = match plugin_dir {
            Ok(pd) => {
                let metadata = match pd.metadata() {
                    Ok(md) => md,
                    Err(err) => {
                        warn!("Failed to get metadata for entry in plugins dir: {}", err);
                        continue
                    },
                };

                if !metadata.is_dir() {
                    continue;
                }

                pd
            }
            Err(_) => continue
        };

        let plugin_dir_path = plugin_dir.path();

        // Path pointing to plugin.toml file
        let plugin_config_path = {
            let mut tmp = plugin_dir_path.clone();
            tmp.push("/plugin.toml");
            tmp
        };

        let plugin_config_file = match read_to_string(&plugin_config_path) {
            Ok(str) => str,
            Err(err) => {
                warn!("Failed to open plugin.toml inside of dir found in plugins dir: {}", err);
                continue;
            }
        };

        let mut toml = match plugin_config_file.parse::<Table>() {
            Ok(toml) => toml,
            Err(err) => {
                warn!("Failed to parse plugin.toml: {}", err);
                continue;
            }
        };

        // Get config section we are interested in
        let Some(config_raw) = toml.remove("plugin") else {
            warn!(
                "Plugin config doesn't contain mandated plugin section, file: {}",
                plugin_config_path.to_string_lossy()
            );
            continue;
        };

        let toml = if toml.len() > 0 {
            Some(toml)
        }
        else {
            None
        };

        if !config_raw.is_table() {
            warn!(
                "Plugin config contains key for \"plugin\", but it isn't a table, file: {}",
                plugin_config_path.to_string_lossy()
            );
            continue;
        };

        // Parse the config
        let config = match toml::from_str::<FlorustServerPluginConfig>(&config_raw.to_string()) {
            Ok(c) => c,
            Err(err) => {
                warn!(
                    "Plugin config (file: {}) couldn't be parsed: {}",
                    plugin_config_path.to_string_lossy(),
                    err
                );
                continue;
            },
        };

        // Get library file path from config
        let plugin_lib_path = {
            let mut tmp = plugin_dir_path.clone();
            tmp.push(config.lib());
            tmp
        };

        // Get manager from library
        let manager_and_data = unsafe {
            let lib = match libloading::Library::new(plugin_lib_path.clone()) {
                Ok(l) => l,
                Err(err) => {
                    warn!(
                        "Failed to open library (path: {}) for plugin (config file: {}) with error: {}",
                        plugin_lib_path.to_string_lossy(),
                        plugin_config_path.to_string_lossy(),
                        err
                    );
                    continue;
                },
            };

            match config.data_type() {
                "i64" => {
                    let create_func_name = if let Some(name) = config.create_func() {
                        name
                    }
                    else {
                        "create_iinteger_data_source_manager"
                    };

                    let create_func: libloading::Symbol<CreateIIntegerDataSourceManager> = match lib.get(create_func_name.as_bytes()) {
                        Ok(m) => m,
                        Err(err) => {
                            warn!(
                                "Failed to retrieve create function ({}) for plugin (path: {}) with error: {}",
                                create_func_name,
                                plugin_dir_path.to_string_lossy(),
                                err
                            );
                            continue;
                        },
                    };

                    match *create_func(Box::new(toml)) {
                        Ok(m) => Box::new(
                            IIntegerManagerAndData::new(m, config.max_data())
                        ) as BoxedManagerAndData,
                        Err(err) => {
                            warn!("Failed to create manager for plugin (path: {}) with error: {}", plugin_dir_path.to_string_lossy(), err);
                            continue;
                        },
                    }
                },
                "u64" => {
                    let create_func_name = if let Some(name) = config.create_func() {
                        name
                    }
                    else {
                        "create_uinteger_data_source_manager"
                    };

                    let create_func: libloading::Symbol<CreateUIntegerDataSourceManager> = match lib.get(create_func_name.as_bytes()) {
                        Ok(m) => m,
                        Err(err) => {
                            warn!(
                                "Failed to retrieve create function ({}) for plugin (path: {}) with error: {}",
                                create_func_name,
                                plugin_dir_path.to_string_lossy(),
                                err
                            );
                            continue;
                        },
                    };

                    match *create_func(Box::new(toml)) {
                        Ok(m) => Box::new(
                            UIntegerManagerAndData::new(m, config.max_data())
                        ),
                        Err(err) => {
                            warn!("Failed to create manager for plugin (path: {}) with error: {}", plugin_dir_path.to_string_lossy(), err);
                            continue;
                        },
                    }
                },
                "f64" => {
                    let create_func_name = if let Some(name) = config.create_func() {
                        name
                    }
                    else {
                        "create_float_data_source_manager"
                    };

                    let create_func: libloading::Symbol<CreateFloatDataSourceManager> = match lib.get(create_func_name.as_bytes()) {
                        Ok(m) => m,
                        Err(err) => {
                            warn!(
                                "Failed to retrieve create function ({}) for plugin (path: {}) with error: {}",
                                create_func_name,
                                plugin_dir_path.to_string_lossy(),
                                err
                            );
                            continue;
                        },
                    };

                    match *create_func(Box::new(toml)) {
                        Ok(m) => Box::new(
                            FloatManagerAndData::new(m, config.max_data())
                        ),
                        Err(err) => {
                            warn!("Failed to create manager for plugin (path: {}) with error: {}", plugin_dir_path.to_string_lossy(), err);
                            continue;
                        },
                    }
                },
                _ => unreachable!("data type config value no longer valid despite being valid a bit ago")
            }
        };

        plugins.push(manager_and_data);
        info!("Loaded plugin: {}", plugin_dir_path.to_string_lossy());
    }

    plugins
}
