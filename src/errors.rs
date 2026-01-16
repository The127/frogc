#[derive(Debug)]
pub struct WrapError {
    inner: Box<dyn std::error::Error + Send + Sync>,
    msg: String,
}

impl WrapError {
    pub fn wrapper<E: std::error::Error + Send + Sync + 'static>(
        msg: &str,
    ) -> impl FnOnce(E) -> WrapError {
        move |inner| WrapError {
            inner: Box::new(inner),
            msg: msg.to_string(),
        }
    }
}

impl std::fmt::Display for WrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.msg, self.inner)
    }
}

impl std::error::Error for WrapError {}

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
