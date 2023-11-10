use rocket::FromForm;

pub mod server;

use serde::{Serialize, Deserialize};


#[derive(FromForm, Serialize, Deserialize)]
pub struct UploadedData {
    pub data: Vec<u8>
}
