use std::{collections::HashMap, result};

use florust_common::{server_plugin::{IIntegerDataSourceManager, UIntegerDataSourceManager, FloatDataSourceManager}, FlorustServerPluginError};
use rocket::{async_trait, tokio::sync::RwLock, serde::{Serialize, Deserialize}};
use thiserror::Error;

use crate::circular_vec::CircularVec;

enum DataSourceStatus<T> where T: Send + Sync {
    Registered(CircularVec<T>),
    RegisteredNoData,
    Deregistered(CircularVec<T>)
}

impl<T> DataSourceStatus<T> where T: Send + Sync {
    fn is_registered(&self) -> bool {
        match self {
            Self::Registered(_) | Self::RegisteredNoData => true,
            Self::Deregistered(_) => false
        }
    }

    fn data_or_err<O: FnOnce() -> ManagerAndDataError>(&self, op: O) -> Result<&CircularVec<T>> {
        match self {
            Self::Registered(data) | Self::Deregistered(data) => Ok(data),
            Self::RegisteredNoData => Err(op())
        }
    }
}

type LoggedData<T> = RwLock<DataSourceStatus<T>>;

type IIntegerDataManager = Box<dyn IIntegerDataSourceManager>;
type IIntegerLoggedData = LoggedData<i64>;

type UIntegerDataManager = Box<dyn UIntegerDataSourceManager>;
type UIntegerLoggedData = LoggedData<u64>;

type FloatDataManager = Box<dyn FloatDataSourceManager>;
type FloatLoggedData = LoggedData<f64>;

pub enum DataType {
    IInteger(i64),
    UInteger(u64),
    Float(f64)
}

#[derive(Serialize, Deserialize, Error, Debug)]
#[serde(crate = "rocket::serde")]
pub enum ManagerAndDataError {
    #[error("Data source manager returned error: {0}")]
    DataSourceManager(FlorustServerPluginError),
    #[error("Attempted to access data from a data source but it has no reported data")]
    NoData,
    #[error("Attempted to access data from a data source but an out of bounds index was used")]
    IndexOutOfBounds
}

pub type Result<T> = result::Result<T, ManagerAndDataError>;

#[async_trait]
pub trait ManagerAndData: Send + Sync {
    fn manager_id(&self) -> &'static str;

    async fn register(&self, id: String) -> Result<()>;

    async fn register_with_data(&self, id: String, data: &[u8]) -> Result<()>;

    async fn deregister(&self, id: &str) -> Result<()>;

    async fn deregister_with_data(&self, id: &str, data: &[u8]) -> Result<()>;

    async fn update_data(&self, id: &str, data: &[u8]) -> Result<()>;

    async fn get_data(&self, id: &str, index: usize) -> Result<DataType>;
}

pub struct IIntegerManagerAndData {
    manager: IIntegerDataManager,
    logged_data: RwLock<HashMap<String, IIntegerLoggedData>>,
    max_logged_data_size: usize
}

pub struct UIntegerManagerAndData {
    manager: UIntegerDataManager,
    logged_data: RwLock<HashMap<String, UIntegerLoggedData>>,
    max_logged_data_size: usize
}

pub struct FloatManagerAndData {
    manager: FloatDataManager,
    logged_data: RwLock<HashMap<String, FloatLoggedData>>,
    max_logged_data_size: usize
}

macro_rules! manager_and_data_impl {
    ($impl_for:ident, $data_manager:ty, $data_type:path) => {
        impl $impl_for {
            pub fn new(manager: $data_manager, max_logged_data_size: usize) -> $impl_for {
                $impl_for {
                    manager,
                    logged_data: RwLock::new(HashMap::new()),
                    max_logged_data_size
                }
            }
        }

        #[async_trait]
        impl ManagerAndData for $impl_for {
            fn manager_id(&self) -> &'static str {
                self.manager_id()
            }

            async fn register(&self, id: String) -> Result<()> {
                let mut lock = self.logged_data.write().await;
                match lock.get(&id) {
                    Some(data_source) => {
                        let mut data_source = data_source.write().await;

                        if data_source.is_registered() {
                            return Err(
                                ManagerAndDataError::DataSourceManager(
                                    FlorustServerPluginError::DataSourceAlreadyExists(id)
                                )
                            )
                        }

                        self.manager.register(id).await.map_err(|err| {
                            ManagerAndDataError::DataSourceManager(
                                FlorustServerPluginError::DataSourceManager(err)
                            )
                        })?;
                        *data_source = DataSourceStatus::RegisteredNoData;
                    }
                    None => {
                        self.manager.register(id.clone()).await.map_err(|err| {
                            ManagerAndDataError::DataSourceManager(
                                FlorustServerPluginError::DataSourceManager(err)
                            )
                        })?;
                        lock.insert(id, RwLock::new(DataSourceStatus::RegisteredNoData));
                    }
                }

                Ok(())
            }

            async fn register_with_data(&self, id: String, data: &[u8]) -> Result<()> {
                let mut lock = self.logged_data.write().await;
                match lock.get(&id) {
                    Some(data_source) => {
                        let mut data_source = data_source.write().await;

                        if data_source.is_registered() {
                            return Err(
                                ManagerAndDataError::DataSourceManager(
                                    FlorustServerPluginError::DataSourceAlreadyExists(id)
                                )
                            )
                        }

                        self.manager.register_with_data(id, data).await.map_err(|err| {
                            ManagerAndDataError::DataSourceManager(
                                FlorustServerPluginError::DataSourceManager(err)
                            )
                        })?;
                        *data_source = DataSourceStatus::RegisteredNoData;
                    }
                    None => {
                        self.manager.register_with_data(id.clone(), data).await.map_err(|err| {
                            ManagerAndDataError::DataSourceManager(
                                FlorustServerPluginError::DataSourceManager(err)
                            )
                        })?;
                        lock.insert(id, RwLock::new(DataSourceStatus::RegisteredNoData));
                    }
                }

                Ok(())
            }

            async fn deregister(&self, id: &str) -> Result<()> {
                let mut lock = self.logged_data.write().await;
                let mut status = lock
                    .get(id)
                    .ok_or(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(id.to_string())
                        )
                    )?
                    .write().await;

                if !status.is_registered() {
                    return Err(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceAlreadyDeregistered(id.to_string())
                        )
                    );
                }

                self.manager.deregister(&id).await
                    .map_err(|err| {
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManager(err)
                        )
                    })?;

                let tmp = std::mem::replace(&mut *status, DataSourceStatus::RegisteredNoData);
                *status = match tmp {
                    DataSourceStatus::Registered(data) => DataSourceStatus::Deregistered(data),
                    DataSourceStatus::RegisteredNoData => DataSourceStatus::RegisteredNoData,
                    DataSourceStatus::Deregistered(_) => unreachable!("DataSourceStatus is Deregistered despite check saying it isn't.")
                };

                if let DataSourceStatus::RegisteredNoData = *status {
                    drop(status);
                    lock.remove(id);
                }

                Ok(())
            }

            async fn deregister_with_data(&self, id: &str, data: &[u8]) -> Result<()> {
                let mut lock = self.logged_data.write().await;
                let mut status = lock
                    .get(id)
                    .ok_or(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManagerDoesntExist(id.to_string())
                        )
                    )?
                    .write().await;

                if !status.is_registered() {
                    return Err(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceAlreadyDeregistered(id.to_string())
                        )
                    );
                }

                self.manager.deregister_with_data(&id, data).await
                    .map_err(|err| {
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceManager(err)
                        )
                    })?;

                let tmp = std::mem::replace(&mut *status, DataSourceStatus::RegisteredNoData);
                *status = match tmp {
                    DataSourceStatus::Registered(data) => DataSourceStatus::Deregistered(data),
                    DataSourceStatus::RegisteredNoData => DataSourceStatus::RegisteredNoData,
                    DataSourceStatus::Deregistered(_) => unreachable!("DataSourceStatus is Deregistered despite check saying it isn't.")
                };

                if let DataSourceStatus::RegisteredNoData = *status {
                    drop(status);
                    lock.remove(id);
                }

                Ok(())
            }

            async fn update_data(&self, id: &str , data: &[u8]) -> Result<()> {
                let lock = self.logged_data.read().await;

                let mut data_source = lock
                    .get(id)
                    .ok_or(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceDoesntExist(id.to_string())
                        )
                    )?
                    .write()
                    .await;

                let val = self.manager.update_data(id, data).await.map_err(|e| {
                    ManagerAndDataError::DataSourceManager(
                        FlorustServerPluginError::DataSourceManager(e)
                    )
                })?;

                match &mut *data_source {
                    DataSourceStatus::RegisteredNoData => {
                        let mut logged_data = CircularVec::new(self.max_logged_data_size);
                        logged_data.append(val);
                        *data_source = DataSourceStatus::Registered(logged_data);
                    },
                    DataSourceStatus::Registered(logged_data) => {
                        logged_data.append(val);
                    },
                    DataSourceStatus::Deregistered(_) => return Err(
                        ManagerAndDataError::DataSourceManager(
                            FlorustServerPluginError::DataSourceDoesntExist(id.to_string())
                        )
                    ),
                }

                Ok(())
            }

            async fn get_data(&self, id: &str, index: usize) -> Result<DataType> {
                Ok(
                    $data_type(
                        *self.logged_data.read().await
                            .get(id)
                            .ok_or(
                                ManagerAndDataError::DataSourceManager(
                                    FlorustServerPluginError::DataSourceDoesntExist(id.to_string())
                                )
                            )?
                            .read().await
                            .data_or_err(|| ManagerAndDataError::NoData)?
                            .get(index)
                            .ok_or(ManagerAndDataError::IndexOutOfBounds)?
                    )
                )
            }
        }
    };
}

manager_and_data_impl!(IIntegerManagerAndData, IIntegerDataManager, DataType::IInteger);
manager_and_data_impl!(UIntegerManagerAndData, UIntegerDataManager, DataType::UInteger);
manager_and_data_impl!(FloatManagerAndData, FloatDataManager, DataType::Float);
