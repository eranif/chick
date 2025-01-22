use serde::Serialize;

#[derive(Default, Debug, Hash, Serialize)]
pub struct DpkgRecord {
    pub package: String,
    pub status: String,
    pub version: String,
}