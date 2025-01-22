use std::sync::Arc;
use oci_client::Client;

pub struct AppState {
    pub oci_client: Arc<Client>,
}