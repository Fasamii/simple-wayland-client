pub mod client;
pub mod dispatch;
pub mod error;

use std::io::Write;
use std::process::exit;

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
    let window_idx = res.unwrap();

    // To creation more than one window just call create_window() again
    let res = client.create_window("meow", "app");
    let _window = res.unwrap();

    let (width, height) = (
        client.globals.windows.get(window_idx).unwrap().window_width,
        client
            .globals
            .windows
            .get(window_idx)
            .unwrap()
            .window_height,
    );

    let mut buff: Vec<u8> =
        vec![0; (width * height * client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap()) as usize];
    loop {
        println!("\t\t\tbefore");
        client.dispatch().unwrap();
        println!("\t\t\tafter");
        let (width, height) = (
            client.globals.windows.get(window_idx).unwrap().window_width,
            client
                .globals
                .windows
                .get(window_idx)
                .unwrap()
                .window_height,
        );

        buff.resize(
            (width * height * client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap()) as usize,
            0,
        );
        for (idx, chunk) in buff
            .chunks_mut(client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap() as usize)
            .enumerate()
        {
            if (idx) == (width * height / 2) as usize {
                chunk[0] = 255;
                chunk[1] = 255;
                chunk[2] = 255;
                chunk[3] = 255;
            } else if (idx) > (width * height / 2) as usize {
                chunk[0] = 255;
                chunk[1] = 80;
                chunk[2] = 80;
                chunk[3] = 100;
            } else {
                chunk[0] = 90;
                chunk[1] = 90;
                chunk[2] = 255;
                chunk[3] = 100;
            }
        }

        client
            .globals
            .windows
            .get_mut(window_idx)
            .unwrap()
            .file
            .write(buff.as_slice())
            .unwrap();
    }
}
