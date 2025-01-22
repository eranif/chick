#[derive(Debug)]
pub struct ImagePullError {
    pub message: String,
}

impl std::fmt::Display for ImagePullError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImagePullError {}