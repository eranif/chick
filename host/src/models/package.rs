use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Hash, Serialize, Deserialize)]
pub struct DpkgRecord {
    pub package: String,
    pub status: String,
    pub version: String,
}