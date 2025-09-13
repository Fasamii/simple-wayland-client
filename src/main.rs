pub mod client;
pub mod dispatch;
pub mod error;

use std::io::Write;
use std::process;

pub use client::Client;
pub use client::State;
pub use client::Window;
pub use error::ClientError;
use rand::Rng;

const DEFAULT_WINDOW_WIDTH: i32 = 480;
const DEFAULT_WINDOW_HEIGHT: i32 = 480;
const DEFAULT_PIXEL_FORMAT: wayland_client::protocol::wl_shm::Format =
    wayland_client::protocol::wl_shm::Format::Argb8888;

fn main() {
    let mut client = Client::new().unwrap();
    let window_idx = client.create_window("woah", "app").unwrap();
    let (width, height) = (
        client.globals.windows.get(window_idx).unwrap().width,
        client
            .globals
            .windows
            .get(window_idx)
            .unwrap()
            .height,
    );

    let mut buff: Vec<u8> =
        vec![0; (width * height * client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap()) as usize];
    loop {
        match client.dispatch() {
            Ok(_) => (),
            Err(_) => process::exit(1),
        }

        for window in client.globals.windows.iter_mut() {
            let (width, height) = (window.width, window.height);
            buff.resize(
                (width * height * client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap()) as usize,
                0,
            );
            for (idx, chunk) in buff
                .chunks_mut(client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap() as usize)
                .enumerate()
            {
                if (idx) > (width * height / 2) as usize {
                    chunk[0] = 200;
                    chunk[1] = 20;
                    chunk[2] = 200;
                    chunk[3] = 255;
                } else {
                    chunk[0] = 90;
                    chunk[1] = 90;
                    chunk[2] = 255;
                    chunk[3] = 255;
                }
            }

            window.file.write(buff.as_slice()).unwrap();
        }
    }
}
