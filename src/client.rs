use super::error::{ClientError, ClientErrorKind};
use std::fs::File;
use std::os::fd::{AsFd, BorrowedFd};
use wayland_client::globals;
use wayland_client::{
    Connection, EventQueue,
    protocol::{wl_buffer, wl_compositor, wl_display, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols::wp::presentation_time::client;
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Debug)]
pub struct Client {
    pub connection: Connection,
    pub display: wl_display::WlDisplay,
    pub queue: EventQueue<Globals>,

    pub globals: Globals,
}

#[derive(Debug)]
pub struct Globals {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub shm: Option<wl_shm::WlShm>,

    pub windows: Vec<Window>,
}
#[derive(Debug)]
pub struct Window {
    pub surface: wl_surface::WlSurface,
    pub xdg_surface: xdg_surface::XdgSurface,
    pub xdg_toplevel: xdg_toplevel::XdgToplevel,

    pub pool: wl_shm_pool::WlShmPool,
    pub file: File,
    pub buffer: wl_buffer::WlBuffer,

    pub window_width: i32,
    pub window_height: i32,
    pub buffer_width: i32,
    pub buffer_height: i32,
}

// TODO: check if you still need that and optionally remove
// fn create_tempfile_with_size(size: i32) -> Result<File, ClientError> {
//     let file = tempfile::tempfile()?;
//     file.set_len(size as u64)?;
//     return Ok(file);
// }

fn bytes_per_pixel(fmt: wl_shm::Format) -> Result<i32, ()> {
    match fmt {
        wl_shm::Format::Argb8888
        | wl_shm::Format::Xrgb8888
        | wl_shm::Format::Abgr8888
        | wl_shm::Format::Xbgr8888 => Ok(4),

        wl_shm::Format::Argb4444
        | wl_shm::Format::Xrgb4444
        | wl_shm::Format::Argb1555
        | wl_shm::Format::Xrgb1555
        | wl_shm::Format::Rgb565 => Ok(2),

        wl_shm::Format::Rgb888 => Ok(3),
        _ => Err(()),
    }
}

impl Client {
    pub fn new() -> Result<Self, ClientError> {
        let connection = Connection::connect_to_env()?;
        let display = connection.display();
        let mut queue = connection.new_event_queue();
        let qhandle = queue.handle();

        display.get_registry(&qhandle, ());

        let mut globals = Globals {
            compositor: None,
            xdg_wm_base: None,
            shm: None,
            windows: Vec::new(),
        };

        queue.roundtrip(&mut globals)?;

        let client = Client {
            connection,
            display,
            queue,
            globals,
        };

        Ok(client)
    }

    pub fn create_window(&mut self, title: &str, id: &str) -> Result<usize, ClientError> {
        let qhandle = self.queue.handle();
        let surface = if let Some(compositor) = &self.globals.compositor {
            compositor.create_surface(&qhandle, ())
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Surface,
                message: "Failed to initialize wl_surface (wl_compositor not available)"
                    .to_string(),
            });
        };
        let xdg_surface = if let Some(xdg_wm_base) = &self.globals.xdg_wm_base {
            xdg_wm_base.get_xdg_surface(&surface, &qhandle, ())
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::XdgSurface,
                message: "Failed to initialize xdg_surface (xdg_wm_base not available)".to_string(),
            });
        };

        let xdg_toplevel = xdg_surface.get_toplevel(&qhandle, ());

        xdg_toplevel.set_title(title.to_string());
        xdg_toplevel.set_app_id(id.to_string());

        // TODO: read docs about configure and fix if you find any mistakes here
        surface.commit(); // <- This line triggers configure event
        self.queue.blocking_dispatch(&mut self.globals)?; // <- this waits for server to
        // configure widow

        let (width, height) = (super::DEFAULT_WINDOW_WIDTH, super::DEFAULT_WINDOW_HEIGHT);
        let pixel_format = super::DEFAULT_PIXEL_FORMAT;

        let pixel_size = match bytes_per_pixel(pixel_format) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(ClientError::Initialization {
                    kind: ClientErrorKind::Pixel,
                    message: "Pixel format not found".to_string(),
                });
            }
        };

        let stride = width * pixel_size;
        let size = stride * height;

        let file = tempfile::tempfile()?;
        file.set_len(size as u64)?;

        let pool = if let Some(shm) = &self.globals.shm {
            shm.create_pool(BorrowedFd::from(file.as_fd()), size, &qhandle, ())
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Pool,
                message: "Failed to initialize wl_pool (wl_shm not available)".to_string(),
            });
        };

        let buffer = pool.create_buffer(0, width, height, stride, pixel_format, &qhandle, ());

        // TODO: check if you have to do that here
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, width, height);
        surface.commit();

        let window = Window {
            surface,
            xdg_surface,
            xdg_toplevel,
            pool,
            file,
            buffer,
            window_width: width,
            window_height: height,
            buffer_width: width,
            buffer_height: height,
        };

        self.globals.windows.push(window);
        Ok(self.globals.windows.len())
    }

    pub fn resize_buffer() {
        todo!()
    }

    pub fn dispatch(&mut self) {
        loop {
            self.queue.blocking_dispatch(&mut self.globals);
        }
        todo!("Implement dipatching for Client struct and globalse then for all child windows");
    }
}
