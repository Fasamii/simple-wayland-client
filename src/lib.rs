mod client;
mod dispatch;
mod error;

const DEFAULT_PIXEL_FORMAT: wayland_client::protocol::wl_shm::Format =
    wayland_client::protocol::wl_shm::Format::Argb8888;

pub use client::Client;
pub use client::State;
pub use client::Window;

pub use error::ClientError;
pub use error::ClientErrorKind;
