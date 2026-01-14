#[derive(Debug)]
pub enum ContainerError {
    AlreadyExists,
    NotFound,
    InvalidState(String),
    WrappedError(Box<dyn std::error::Error + Send + Sync>),
}

impl ContainerError {
    pub fn wrap<E: std::error::Error + Send + Sync + 'static>(e: E) -> Self {
        ContainerError::WrappedError(Box::new(e))
    }
}

impl std::fmt::Display for ContainerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContainerError::AlreadyExists => write!(f, "Container already exists"),
            ContainerError::NotFound => write!(f, "Container not found"),
            ContainerError::InvalidState(s) => write!(f, "Invalid state: {}", s),
            ContainerError::WrappedError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for ContainerError {}
