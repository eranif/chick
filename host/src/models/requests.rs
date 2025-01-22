use serde::{Deserialize, Serialize};
use super::package::DpkgRecord;

#[derive(Deserialize)]
pub struct InspectRequest {
    pub image: String,
}

#[derive(Serialize)]
pub struct InspectResponse {
    pub image: String,
    pub layers: Vec<Layer>,
}

#[derive(Serialize)]
pub struct Layer {
    pub layer: String,
    pub packages: Vec<DpkgRecord>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}