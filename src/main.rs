pub mod client;
pub mod dispatch;
pub mod error;

pub use client::Client;
pub use client::Globals;
pub use client::Window;
pub use error::ClientError;

const DEFAULT_WINDOW_WIDTH: i32 = 480;
const DEFAULT_WINDOW_HEIGHT: i32 = 480;
const DEFAULT_PIXEL_FORMAT: wayland_client::protocol::wl_shm::Format =
    wayland_client::protocol::wl_shm::Format::Argb8888;

fn main() {
    // NOTE: this is only example API (not final version)

    // First you need to create global client for your app
    let mut client = Client::new().unwrap();

    // To create window use
    let res = client.create_window("woah", "app");
    let _window = res.unwrap();

    // To creation more than one window just call create_window() again
    let res = client.create_window("meow", "app");
    let _window = res.unwrap();

    loop {
        client.dispatch();
    }
}
