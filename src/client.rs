use super::error::{ClientError, ClientErrorKind};
use std::fs::File;
use std::io::Seek;
use std::os::fd::{AsFd, BorrowedFd};
use wayland_client::{
    Connection, EventQueue, QueueHandle,
    protocol::{wl_buffer, wl_compositor, wl_display, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Debug)]
pub struct Client {
    pub connection: Connection,
    pub display: wl_display::WlDisplay,
    pub queue: EventQueue<State>,
    pub globals: State,
}
#[derive(Debug)]
pub struct State {
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
    pub buffers: Vec<Buffer>,

    pub width: i32,
    pub height: i32,

    pub needs_resizing: bool,
}

#[derive(Debug)]
pub struct Buffer {
    pub data: wl_buffer::WlBuffer,
    pub used: bool,
    pub destroy: bool,

    pub width: i32,
    pub height: i32,
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

        let mut globals = State {
            compositor: None,
            xdg_wm_base: None,
            shm: None,
            windows: Vec::new(),
        };

        // NOTE: that was shitty idea wasabi ~ fck you...
        queue.roundtrip(&mut globals)?;

        let client = Client {
            connection,
            display,
            queue,
            globals,
        };

        Ok(client)
    }

    pub fn dispatch(&mut self) -> Result<(), ClientError> {
        self.queue.blocking_dispatch(&mut self.globals)?;
        Ok(())
    }

    pub fn create_window(&mut self, title: &str, id: &str) -> Result<usize, ClientError> {
        let qhandle = self.queue.handle();
        let surface = State::create_surface(&self.globals, &qhandle)?;
        let xdg_surface = State::create_xdg_surface(&self.globals, &surface, &qhandle)?;
        let xdg_toplevel = xdg_surface.get_toplevel(&qhandle, self.globals.windows.len());

        xdg_toplevel.set_title(title.to_string());
        xdg_toplevel.set_app_id(id.to_string());

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

        let pool = State::create_pool(&self.globals, &qhandle, &file, size)?;

        let buffer = pool.create_buffer(
            0,
            width,
            height,
            stride,
            pixel_format,
            &qhandle,
            self.globals.windows.len(),
        ); // TODO:
        // create offset for double buffering (I think it should be buffer size but not sure) / or remove

        let mut buffer = Buffer {
            data: buffer,
            used: false,
            destroy: false,
            width: width,
            height: height,
        };

        // TODO: check if you have to do that here
        surface.attach(Some(&buffer.data), 0, 0);

        surface.damage_buffer(0, 0, width, height);
        surface.commit();
        buffer.used = true;

        let window = Window {
            surface,
            xdg_surface,
            xdg_toplevel,
            pool,
            file,
            width: width,
            height: height,
            buffers: vec![buffer],
            needs_resizing: true,
        };

        self.globals.windows.push(window);
        Ok(self.globals.windows.len() - 1)
    }
}

impl State {
    pub fn dispatch() {
        todo!()
    }

    pub fn resize_buffer(
        &mut self,
        qhandle: &QueueHandle<State>,
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

        let (window_width, window_height) = {
            let window = self.windows.get(idx).unwrap();
            (window.width, window.height)
        };

        let stride = window_width * pixel_size;
        let size = stride * window_height;

        let width = window_width;
        let height = window_height;

        let mut file = { &self.windows.get_mut(idx).unwrap().file };
        file.set_len((size) as u64)?;
        file.rewind()?;

        let mut pool = Self::create_pool(&self, &qhandle, &self.windows.get(idx).unwrap().file, size)?;

        let buffer = pool.create_buffer(
            0,
            window_width,
            window_height,
            stride,
            pixel_format,
            &qhandle,
            idx,
        );

        let buffer = Buffer {
            data: buffer,
            destroy: false,
            used: true,
            width: window_width,
            height: window_height,
        };

        for buffer in &mut self.windows.get_mut(idx).unwrap().buffers {
            buffer.destroy = true;
        }

        self.windows
            .get_mut(idx)
            .unwrap()
            .surface
            .attach(Some(&buffer.data), 0, 0); 

        self.windows
            .get_mut(idx)
            .unwrap()
            .surface
            .damage_buffer(0, 0, width, height);

        self.windows.get_mut(idx).unwrap().surface.commit();

        let _old_pool = std::mem::replace(&mut self.windows.get_mut(idx).unwrap().pool, pool);

        self.windows.get_mut(idx).unwrap().buffers.push(buffer);
        self.windows.get_mut(idx).unwrap().needs_resizing = false;

        Ok(())
    }

    fn create_pool(
        &self,
        qhandle: &QueueHandle<State>,
        file: &File,
        size: i32,
    ) -> Result<wl_shm_pool::WlShmPool, ClientError> {
        if let Some(shm) = &self.shm {
            return Ok(shm.create_pool(BorrowedFd::from(file.as_fd()), size, qhandle, ()));
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Pool,
                message: "Failed to initialize wl_shm_pool (wl_shm not available)".to_string(),
            });
        }
    }

    fn create_surface(
        &self,
        qhandle: &QueueHandle<State>,
    ) -> Result<wl_surface::WlSurface, ClientError> {
        if let Some(compositor) = &self.compositor {
            return Ok(compositor.create_surface(&qhandle, ()));
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::Surface,
                message: "Failed to initialize wl_surface (wl_compositor not available)"
                    .to_string(),
            });
        };
    }

    fn create_xdg_surface(
        &self,
        surface: &wl_surface::WlSurface,
        qhandle: &QueueHandle<State>,
    ) -> Result<xdg_surface::XdgSurface, ClientError> {
        if let Some(xdg_wm_base) = &self.xdg_wm_base {
            return Ok(xdg_wm_base.get_xdg_surface(surface, qhandle, self.windows.len()));
        } else {
            return Err(ClientError::Initialization {
                kind: ClientErrorKind::XdgSurface,
                message: "Failed to initialize xdg_surface (xdg_wm_base not available)".to_string(),
            });
        };
    }
}
