use async_trait::async_trait;

#[async_trait]
pub trait AtomicDataSourceManager {
    fn class(&self) -> &'static str;

    async fn register(&self, id: String);

    async fn unregister(&self, id: &str);

    async fn update_data(&self, id: &str);
}

#[async_trait]
pub trait DataSourceManager {
    fn class(&self) -> &'static str;

    async fn register(&self, id: String);

    async fn unregister(&self, id: &str);

    async fn update_data(&mut self, id: &str);
}

pub type CreateAtomicDataSourceManager = unsafe extern fn() -> Box<Box<dyn AtomicDataSourceManager>>;
pub type CreateDataSourceManager = unsafe extern fn() -> Box<Box<dyn DataSourceManager>>;