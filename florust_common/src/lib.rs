use rocket::FromForm;

pub mod server_plugin;
pub mod server_data_source_error;

#[derive(FromForm)]
pub struct UploadedData {
    pub data: Vec<(String, Vec<u8>)>
}
