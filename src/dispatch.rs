use crate::client::{Client, Globals};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_buffer, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface},
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

impl Dispatch<wl_registry::WlRegistry, ()> for Globals {
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

impl Dispatch<wl_compositor::WlCompositor, ()> for Globals {
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

impl Dispatch<wl_shm::WlShm, ()> for Globals {
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

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for Globals {
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

impl Dispatch<wl_surface::WlSurface, ()> for Globals {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        event: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!(". Recivied (WL_SURFACE) Event : {event:?}");
    }
}

impl Dispatch<xdg_surface::XdgSurface, usize> for Globals {
    fn event(
        state: &mut Self,
        proxy: &xdg_surface::XdgSurface,
        event: <xdg_surface::XdgSurface as wayland_client::Proxy>::Event,
        data: &usize,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!(". Recivied (XDG_SURFACE) Event : {event:?}");
        if let xdg_surface::Event::Configure { serial } = event {
            // Client::resize_buffer(state, *idx);
            proxy.ack_configure(serial); // FIX: you should probable first resize buffer if it
            // truly needs that and then ack.
        }
    }
}

impl wayland_client::Dispatch<xdg_toplevel::XdgToplevel, usize> for Globals {
    fn event(
        state: &mut Globals,
        _proxy: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        window_id: &usize,
        _conn: &Connection,
        _qh: &QueueHandle<Globals>,
    ) {
        if let Some(window) = state.windows.get_mut(*window_id) {
            match event {
                xdg_toplevel::Event::Configure { width, height, .. } => {
                    if width > 0 {
                        window.window_width = width;
                    }
                    if height > 0 {
                        window.window_height = height;
                    }
                    window.needs_ressising = true;
                }
                xdg_toplevel::Event::Close => {
                    // mark window for close, or remove from Vec
                    // TODO: handle close more gracefuly
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for Globals {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        todo!()
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for Globals {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        event: <wl_buffer::WlBuffer as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("Recivied (WL_BUFFER) Event : {event:?}");
    }
}
