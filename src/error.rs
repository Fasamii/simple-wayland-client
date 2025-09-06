#[derive(Debug)]
pub enum ClientError {
    Connection(wayland_client::ConnectError),
    Dispatch(wayland_client::DispatchError),
    Initialization {
        kind: ClientErrorKind,
        message: String,
    },
}

#[derive(Debug)]
pub enum ClientErrorKind {
    Pool,
    File,
    Pixel,
    Surface,
    XdgSurface,
    XdgTopLevel,
}

impl ClientError {
    pub fn kind(&self) -> Option<&ClientErrorKind> {
        match self {
            ClientError::Connection(_) => None,
            ClientError::Dispatch(_) => None,
            ClientError::Initialization { kind, message: _ } => Some(kind),
        }
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Connection(err) => write!(f, "Failed to connect to Wayland: {err}"),
            ClientError::Dispatch(err) => write!(f, "Failed to dispatch: {err}"),
            ClientError::Initialization { message, kind: _ } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::Connection(err) => Some(err),
            ClientError::Dispatch(err) => Some(err),
            ClientError::Initialization { .. } => None,
        }
    }
}

impl From<wayland_client::ConnectError> for ClientError {
    fn from(err: wayland_client::ConnectError) -> Self {
        ClientError::Connection(err)
    }
}

impl From<wayland_client::DispatchError> for ClientError {
    fn from(err: wayland_client::DispatchError) -> Self {
        ClientError::Dispatch(err)
    }
}

impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::Initialization {
            kind: ClientErrorKind::File,
            message: format!("Failed to create tempfile : {err}"),
        }
    }
}
