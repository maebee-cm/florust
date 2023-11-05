use std::collections::HashMap;

use florust_common::{server_plugin::{IIntegerDataSourceManager, UIntegerDataSourceManager, FloatDataSourceManager, self}, server_data_source_error::DataSourceManagerError};
use rocket::{async_trait, tokio::sync::RwLock};

use crate::circular_vec::CircularVec;

enum DataSourceStatus<T> where T: Send + Sync {
    RegisteredNoData,
    RegisteredWithData(CircularVec<T>),
    Deregistered
}

impl<T> DataSourceStatus<T> where T: Send + Sync {
    fn is_deregistered(&self) -> bool {
        if let DataSourceStatus::Deregistered = self {
            true
        }
        else {
            false
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

#[async_trait]
pub trait ManagerAndData: Send + Sync {
    async fn register(&self, id: String) -> server_plugin::Result<()>;

    async fn register_with_data(&self, id: String, data: &[u8]) -> server_plugin::Result<()>;

    async fn deregister(&self, id: &str) -> server_plugin::Result<()>;

    async fn deregister_with_data(&self, id: &str, data: &[u8]) -> server_plugin::Result<()>;

    async fn update_data(&self, id: &str, data: &[u8]) -> server_plugin::Result<()>;
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
    ($impl_for:ident, $data_manager:ty) => {
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
            async fn register(&self, id: String) -> server_plugin::Result<()> {
                let mut lock = self.logged_data.write().await;

                if let Some(data_source) = lock.get(&id) {
                    let mut data_source = data_source.write().await;
                    if !data_source.is_deregistered() {
                        return Err(DataSourceManagerError::IdAlreadyExists);
                    }

                    self.manager.register(id).await?;
                    *data_source = DataSourceStatus::RegisteredNoData;
                }
                else {
                    self.manager.register(id.clone()).await?;
                    lock.insert(id, RwLock::new(DataSourceStatus::RegisteredNoData));
                }

                Ok(())
            }

            async fn register_with_data(&self, id: String, data: &[u8]) -> server_plugin::Result<()> {
                let mut lock = self.logged_data.write().await;

                if let Some(data_source) = lock.get(&id) {
                    let mut data_source = data_source.write().await;
                    if !data_source.is_deregistered() {
                        return Err(DataSourceManagerError::IdAlreadyExists);
                    }

                    self.manager.register_with_data(id, data).await?;
                    *data_source = DataSourceStatus::RegisteredNoData;
                }
                else {
                    self.manager.register_with_data(id.clone(), data).await?;
                    lock.insert(id, RwLock::new(DataSourceStatus::RegisteredNoData));
                }

                Ok(())
            }

            async fn deregister(&self, id: &str) -> server_plugin::Result<()> {
                let lock = self.logged_data.read().await;
                
                let mut data_source = lock
                    .get(id)
                    .ok_or(DataSourceManagerError::IdDoesntExist)?
                    .write()
                    .await;

                if data_source.is_deregistered() {
                    return Err(DataSourceManagerError::IdDoesntExist);
                }

                self.manager.deregister(id).await?;
                *data_source = DataSourceStatus::Deregistered;

                Ok(())
            }

            async fn deregister_with_data(&self, id: &str, data: &[u8]) -> server_plugin::Result<()> {
                let lock = self.logged_data.read().await;
                
                let mut data_source = lock
                    .get(id)
                    .ok_or(DataSourceManagerError::IdDoesntExist)?
                    .write()
                    .await;

                if data_source.is_deregistered() {
                    return Err(DataSourceManagerError::IdDoesntExist);
                }

                self.manager.deregister_with_data(id, data).await?;
                *data_source = DataSourceStatus::Deregistered;

                Ok(())
            }

            async fn update_data(&self, id: &str, data: &[u8]) -> server_plugin::Result<()> {
                let lock = self.logged_data.read().await;

                let mut data_source = lock
                    .get(id)
                    .ok_or(DataSourceManagerError::IdDoesntExist)?
                    .write()
                    .await;

                let val = self.manager.update_data(id, data).await?;

                match &mut *data_source {
                    DataSourceStatus::RegisteredNoData => {
                        let mut logged_data = CircularVec::new(self.max_logged_data_size);
                        logged_data.append(val);
                        *data_source = DataSourceStatus::RegisteredWithData(logged_data);
                    },
                    DataSourceStatus::RegisteredWithData(logged_data) => {
                        logged_data.append(val);

                    },
                    DataSourceStatus::Deregistered => return Err(DataSourceManagerError::IdDoesntExist),
                }

                Ok(())
            }
        }
    };
}

manager_and_data_impl!(IIntegerManagerAndData, IIntegerDataManager);
manager_and_data_impl!(UIntegerManagerAndData, UIntegerDataManager);
manager_and_data_impl!(FloatManagerAndData, FloatDataManager);
