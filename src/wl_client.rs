use wayland_client::protocol::wl_display::WlDisplay;
// Top level wayland protocol handlers
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
// Wayland objects
use wayland_client::protocol::{
    wl_buffer,     // Represents a buffer of pixel data
    wl_compositor, // Creates surfaces and regions
    wl_display,    // Root object of the Wayland protocol
    wl_registry,   // Global object registry for discovering interfaces
    wl_shm,        // Shared memory interface for pixel buffers
    wl_shm_pool,   // Pool of shared memory
    wl_surface,    // Rectangular area that can be displayed
};

// XDG shell is a Wayland protocol extension for desktop-style windows
use wayland_protocols::xdg::shell::client::{
    xdg_surface,  // XDG surface - adds window management to wl_surface
    xdg_toplevel, // Top-level window (what users think of as "windows")
    xdg_wm_base,  // Client manager base - entry point for XDG shell
};

#[derive(Debug)]
pub struct Client {
    pub state: State,
    pub queue: EventQueue<State>,
    pub connection: Connection,
    pub display: WlDisplay,
}

#[derive(Debug)]
pub struct State {
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    xdg_wm_base: Option<xdg_wm_base::XdgWmBase>,

    xdg_top_level: Option<xdg_toplevel::XdgToplevel>,
    surface: Option<wl_surface::WlSurface>,
    xdg_surface: Option<xdg_surface::XdgSurface>,

    width: i32,
    height: i32,

    buffer: Option<wl_buffer::WlBuffer>,
    pool: Option<wl_shm_pool::WlShmPool>,
}

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

impl Client {
    pub fn new() -> Result<Self, ClientError> {
        let connection = Connection::connect_to_env()?;
        let display = connection.display();
        let mut queue = connection.new_event_queue();
        let qhandle = queue.handle();

        display.get_registry(&qhandle, ());

        let mut state = State {
            compositor: None,
            shm: None,
            xdg_wm_base: None,
            xdg_top_level: None,
            surface: None,
            xdg_surface: None,
            width: 0,
            height: 0,
            buffer: None,
            pool: None,
        };

        queue.roundtrip(&mut state)?;

        Ok(Client {
            connection,
            display,
            queue,
            state,
        })
    }

    pub fn create_buffer(&mut self) -> Result<(), ClientError> {
        todo!("create buffer");
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

    pub fn dispach() {
        todo!("implement dispatching in this function you should handle resize etc...");
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _user_data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    state.compositor = Some(proxy.bind::<wl_compositor::WlCompositor, _, _>(
                        name,
                        version,
                        qhandle,
                        (),
                    ));
                }
                "wl_shm" => {
                    state.shm = Some(proxy.bind::<wl_shm::WlShm, _, _>(name, version, qhandle, ()));
                }
                "xdg_wm_base" => {
                    state.xdg_wm_base = Some(proxy.bind::<xdg_wm_base::XdgWmBase, _, _>(
                        name,
                        version,
                        qhandle,
                        (),
                    ));
                }
                _ => (),
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        event: <wl_compositor::WlCompositor as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!(". Recivied (COMPOSITOR) event : {event:?}");
    }
}

impl Dispatch<wl_shm::WlShm, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        event: <wl_shm::WlShm as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!(". Recivied (SHM) Event : {event:?}");
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _state: &mut Self,
        proxy: &xdg_wm_base::XdgWmBase,
        event: <xdg_wm_base::XdgWmBase as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            proxy.pong(serial);
            println!(". Recivied ping");
        } else {
            println!(". Recivied (XDG_WM_BASE) Event : {event:?}");
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &xdg_toplevel::XdgToplevel,
        event: <xdg_toplevel::XdgToplevel as wayland_client::Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!(". Recivied (XDG_TOP_LEVEL) Event : {event:?}");
        match event {
            xdg_toplevel::Event::Configure {
                width,
                height,
                states,
            } => todo!(),
            xdg_toplevel::Event::Close => todo!(),
            xdg_toplevel::Event::ConfigureBounds { width, height } => todo!(),
            xdg_toplevel::Event::WmCapabilities { capabilities } => (),
            _ => (),
        };
    }
}
