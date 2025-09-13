use crate::client::State;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{wl_buffer, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _user_data: &(),
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        // TODO: also handle GlobalRemove
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
        }
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<xdg_surface::XdgSurface, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as wayland_client::Proxy>::Event,
        idx: &usize,
        _conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            proxy.ack_configure(serial); // READ: ack_configure function doc
            match state.windows.get(*idx) {
                Some(window) => {
                    if window.needs_resizing {
                        State::resize_buffer(state, &qhandle, *idx).unwrap();
                    }
                }
                _ => (),
            }
        }
    }
}

impl wayland_client::Dispatch<xdg_toplevel::XdgToplevel, usize> for State {
    fn event(
        state: &mut State,
        _proxy: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        window_id: &usize,
        _conn: &Connection,
        _qh: &QueueHandle<State>,
    ) {
        if let Some(window) = state.windows.get_mut(*window_id) {
            match event {
                xdg_toplevel::Event::Configure { width, height, .. } => {
                    window.needs_resizing = if window.width != width || window.height != height {
                        true
                    } else {
                        false
                    };
                    if width > 0 {
                        window.width = width;
                    }
                    if height > 0 {
                        window.height = height;
                    }
                }
                xdg_toplevel::Event::Close => {
                    // TODO: handle close more gracefully
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for State {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_buffer::WlBuffer,
        event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        idx: &usize,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let wl_buffer::Event::Release = event {
            if let Some(window) = state.windows.get_mut(*idx) {
                if let Some(buffer) = window
                    .buffers
                    .iter_mut()
                    .find(|b| b.data.id() == proxy.id())
                {
                    buffer.used = false;
                }
                window
                    .buffers
                    .retain(|buffer| !buffer.destroy || buffer.used);
            }
        }
    }
}
