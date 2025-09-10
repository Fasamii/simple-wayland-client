use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    protocol::{
        wl_buffer, wl_compositor, wl_display, wl_registry, wl_shm, wl_shm_pool, wl_surface,
    },
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

#[derive(Debug)]
pub struct Client {
    pub state: State,
    pub queue: EventQueue<State>,
    pub connection: Connection,
    pub display: WlDisplay,
    pub automatic_resize: bool,
}

#[derive(Debug)]
pub struct State {
    pub compositor: Option<wl_compositor::WlCompositor>,
    pub xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,

    pub surface: Option<wl_surface::WlSurface>,
    pub xdg_surface: Option<xdg_surface::XdgSurface>,
    pub xdg_top_level: Option<xdg_toplevel::XdgToplevel>,

    pub shm: Option<wl_shm::WlShm>,
    pub pool: Option<wl_shm_pool::WlShmPool>,
    pub buffer_file: Option<File>,
    pub buffer: Option<wl_buffer::WlBuffer>,

    pub window_width: i32,
    pub window_height: i32,
    pub buffer_width: i32,
    pub buffer_height: i32,
}

fn create_tempfile_with_size(size: i32) -> Result<File, ClientError> {
    let file = tempfile::tempfile()?;
    file.set_len(size as u64)?;
    return Ok(file);
}

impl Client {
    pub fn new() -> Result<Self, ClientError> {
        let connection = Connection::connect_to_env()?;
        let display = connection.display();
        let mut queue = connection.new_event_queue();
        let qhandle = queue.handle();

        display.get_registry(&qhandle, ());

        let mut state = State {
            window_width: 0,
            window_height: 0,
            buffer_width: 0,
            buffer_height: 0,
            compositor: None,
            xdg_wm_base: None,
            surface: None,
            xdg_surface: None,
            xdg_top_level: None,
            shm: None,
            pool: None,
            buffer_file: None,
            buffer: None,
        };

        queue.roundtrip(&mut state)?;

        Ok(Client {
            connection,
            display,
            queue,
            state,
            automatic_resize: false,
        })
    }

    pub fn create_surface(&mut self) -> Result<(), ClientError> {
        let qhandle = self.queue.handle();

        if self.state.surface.is_none() {
            if let Some(compositor) = &self.state.compositor {
                self.state.surface = Some(compositor.create_surface(&qhandle, ()));
            } else {
                return Err(ClientError::Initialization {
                    kind: ClientErrorKind::Surface,
                    message: "Failed to initialize wl_surface (wl_compositor not available)"
                        .to_string(),
                });
            }
        }

        if self.state.xdg_surface.is_none() {
            if let Some(xdg_wm_base) = &self.state.xdg_wm_base {
                self.state.xdg_surface = Some(xdg_wm_base.get_xdg_surface(
                    &self.state.surface.as_ref().unwrap(),
                    &qhandle,
                    (),
                ));
            } else {
                return Err(ClientError::Initialization {
                    kind: ClientErrorKind::XdgSurface,
                    message: "Failed to initialize xdg_surface (xdg_wm_base not available)"
                        .to_string(),
                });
            }
        }

        if self.state.xdg_top_level.is_none() {
            if let Some(xdg_surface) = &self.state.xdg_surface {
                self.state.xdg_top_level = Some(xdg_surface.get_toplevel(&qhandle, ()));
            } else {
                return Err(ClientError::Initialization {
                    kind: ClientErrorKind::XdgTopLevel,
                    message: "Failed to initialize xdg_top_level (xdg_surface not available)"
                        .to_string(),
                });
            }
        }

        Ok(())
    }

    // TODO: fix this madness
    pub fn create_buffer(&mut self, automatic_resize: bool) -> Result<(), ClientError> {
        self.automatic_resize = automatic_resize;

        let stride = self.state.window_width * 4;
        let size = stride * self.state.window_height;

        let qhandle = self.queue.handle();

        self.state.buffer_file = if let Some(file) = &self.state.buffer_file {
            Some(file.try_clone().unwrap())
        } else {
            Some(create_tempfile_with_size(size)?)
        };

        self.state.pool = if let Some(pool) = &self.state.pool {
            Some(pool.to_owned())
        } else {
            if let Some(shm) = &self.state.shm {
                Some(shm.create_pool(
                    BorrowedFd::from(self.state.buffer_file.as_ref().unwrap().as_fd()),
                    size,
                    &qhandle,
                    (),
                ))
            } else {
                return Err(ClientError::Initialization {
                    kind: ClientErrorKind::Pool,
                    message: "Failed to initialize wl_pool (wl_shm not available)".to_string(),
                });
            }
        };

        self.state.buffer = if let Some(buffer) = &self.state.buffer {
            Some(buffer.to_owned())
        } else {
            Some(self.state.pool.as_ref().unwrap().create_buffer(
                0,
                self.state.window_width,
                self.state.window_height,
                stride,
                wl_shm::Format::Argb8888,
                &qhandle,
                (),
            ))
        };

        self.state.buffer_width = self.state.window_width;
        self.state.buffer_height = self.state.window_height;

        Ok(())
    }

    pub fn dispatch(&mut self) {
        todo!("call create buffer if it is needed")
    }
}
