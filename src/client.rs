// TODO: Fuck this shit. Redo that from scratch later.
// NOTES: Implement Dispatch for Client not for Globals
//          fuck but then you can't init structs in new() normal way (thanks rust (borrow checker
//          really helped here fuck this function <3))
use super::error::{ClientError, ClientErrorKind};
use std::fs::File;
use std::io::Seek;
use std::os::fd::{AsFd, BorrowedFd};
use wayland_client::QueueHandle;
use wayland_client::{
    Connection, EventQueue,
    protocol::{wl_buffer, wl_compositor, wl_display, wl_shm, wl_shm_pool, wl_surface},
};
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

    pub needs_ressising: bool,
}

pub fn bytes_per_pixel(fmt: wl_shm::Format) -> Result<i32, ()> {
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

        wl_shm::Format::Rgb888 => Ok(4),
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

        // NOTE: to avoid this stupid shit store data like queue etc... separately
        // then have separate struct to hold window and globals and make that impl Dispatch :3
        // great idea ~ Wasabi, thanks, i know ~ Wasabi (Love this API :3)
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
            xdg_wm_base.get_xdg_surface(&surface, &qhandle, self.globals.windows.len())
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::XdgSurface,
                message: "Failed to initialize xdg_surface (xdg_wm_base not available)".to_string(),
            });
        };

        let xdg_toplevel = xdg_surface.get_toplevel(&qhandle, self.globals.windows.len()); // NOTE:
        // len not len - 1 since window isn't appended to Vector yet

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
        file.set_len((size) as u64)?; // TODO: add * 2 for double buffering / or remove

        let pool = if let Some(shm) = &self.globals.shm {
            shm.create_pool(BorrowedFd::from(file.as_fd()), size, &qhandle, ()) // TODO:
        // add *2 for double buffering / or remove
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Pool,
                message: "Failed to initialize wl_pool (wl_shm not available)".to_string(),
            });
        };

        let buffer0 = pool.create_buffer(0, width, height, stride, pixel_format, &qhandle, ()); // TODO:
        // create offset for double buffering (I think it should be buffer size but not sure) / or remove

        // TODO: check if you have to do that here
        surface.attach(Some(&buffer0), 0, 0);

        surface.damage_buffer(0, 0, width, height);
        surface.commit();

        let window = Window {
            surface,
            xdg_surface,
            xdg_toplevel,
            pool,
            file,
            buffer: buffer0,
            window_width: width,
            window_height: height,
            buffer_width: width,
            buffer_height: height,
            needs_ressising: false,
        };

        self.globals.windows.push(window);
        Ok(self.globals.windows.len() - 1)
    }

    pub fn dispatch(&mut self) -> Result<(), ClientError> {
        // NOTE: this is only  temporal approach just to make demo running
        #[allow(unused_must_use)]
        self.queue.blocking_dispatch(&mut self.globals);

        Ok(())
    }
}
impl Globals {
    pub fn resize_buffer(
        globals: &mut Globals,
        qhandle: &QueueHandle<Globals>,
        idx: usize,
    ) -> Result<(), ClientError> {
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

        let stride = globals.windows.get(idx).unwrap().window_width * pixel_size;
        let size = stride * globals.windows.get(idx).unwrap().window_height;

        let width = globals.windows.get(idx).unwrap().window_width;
        let height = globals.windows.get(idx).unwrap().window_height;
        println!(
            "Pixel size ({pixel_size}), stride ({stride}), size ({size}), width x heithh ({width}x{height})"
        );

        globals
            .windows
            .get(idx)
            .unwrap()
            .file
            .set_len((size) as u64)?;
        globals.windows.get_mut(idx).unwrap().file.rewind()?;

        let pool = if let Some(shm) = &globals.shm {
            shm.create_pool(
                BorrowedFd::from(globals.windows.get(idx).unwrap().file.as_fd()),
                size,
                &qhandle,
                (),
            )
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Pool,
                message: "Failed to initialize wl_pool (wl_shm not available)".to_string(),
            });
        };

        let buffer = pool.create_buffer(
            0,
            globals.windows.get(idx).unwrap().window_width,
            globals.windows.get(idx).unwrap().window_height,
            stride,
            pixel_format,
            &qhandle,
            (),
        );

        globals
            .windows
            .get_mut(idx)
            .unwrap()
            .surface
            .attach(Some(&buffer), 0, 0);

        globals
            .windows
            .get_mut(idx)
            .unwrap()
            .surface
            .damage_buffer(0, 0, width, height);
        globals.windows.get_mut(idx).unwrap().surface.commit();

        globals.windows.get(idx).unwrap().buffer.destroy();
        globals.windows.get_mut(idx).unwrap().buffer = buffer;

        // NOTE: to not call it again
        globals.windows.get_mut(idx).unwrap().buffer_width =
            globals.windows.get(idx).unwrap().window_width;
        globals.windows.get_mut(idx).unwrap().buffer_height =
            globals.windows.get(idx).unwrap().window_height;
        globals.windows.get_mut(idx).unwrap().needs_ressising = false;

        Ok(())
    }
}
