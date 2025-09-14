pub mod client;
pub mod dispatch;
pub mod error;

use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::process;

pub use client::Client;
pub use client::State;
pub use client::Window;
pub use error::ClientError;

const DEFAULT_PIXEL_FORMAT: wayland_client::protocol::wl_shm::Format =
    wayland_client::protocol::wl_shm::Format::Argb8888;

fn main() {
    let mut client = Client::new().unwrap();

    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();
    let _ = client.create_window("woah", "app").unwrap();

    loop {
        match client.dispatch() {
            Ok(_) => (),
            Err(_) => {
                process::exit(1);
            }
        }

        for (idx, window) in client.globals.windows.iter_mut().enumerate() {
            let (width, height) = (window.width, window.height);
            let mut buff: Vec<u8> = vec![
                0;
                (width * height * client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap())
                    as usize
            ];

            for chunk in
                buff.chunks_mut(client::bytes_per_pixel(DEFAULT_PIXEL_FORMAT).unwrap() as usize)
            {
                if idx % 2 == 0 {
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

            if !window.buffers.get(0).unwrap().used {
                window.file.seek(SeekFrom::Start(0)).unwrap(); // IMPORTANT: check if you need
                // to use seek from start
                window.file.write_all(buff.as_slice()).unwrap(); // IMPORTANT: read docs for
                // write_all and compare it to just write
            }

            window.surface.commit();
        }
    }
}
