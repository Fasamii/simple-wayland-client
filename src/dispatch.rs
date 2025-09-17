use crate::client::State;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface,
    },
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
            match state.windows.get(*idx) {
                Some(window) => {
                    println!(
                        ". ({idx}) ack_configure() needs_resizing({}) to ({}x{})",
                        window.needs_resizing, window.width, window.height
                    );
                    if window.needs_resizing {
                        State::resize_buffer(state, &qhandle, *idx).unwrap();
                    }
                }
                _ => (),
            }
            proxy.ack_configure(serial);
            // state.windows.get(*idx).unwrap().surface.commit();
        }
    }
}

impl wayland_client::Dispatch<xdg_toplevel::XdgToplevel, usize> for State {
    fn event(
        state: &mut State,
        _proxy: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        idx: &usize,
        _conn: &Connection,
        _qhandle: &QueueHandle<State>,
    ) {
        if let Some(window) = state.windows.get_mut(*idx) {
            match event {
                xdg_toplevel::Event::Configure { width, height, .. } => {
                    window.needs_resizing = if (window.width != width || window.height != height)
                        || window.needs_resizing
                    {
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

                    println!(
                        ". ({idx}) ({}x{}) => ({}x{}) ({})",
                        window.width, window.height, width, height, window.needs_resizing
                    );
                }

                xdg_toplevel::Event::Close => {
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
        _event: wl_shm_pool::Event,
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
            println!(". ({idx}) buffer event");
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

impl Dispatch<wl_callback::WlCallback, usize> for State {
    fn event(
        state: &mut Self,
        proxy: &wl_callback::WlCallback,
        event: <wl_callback::WlCallback as Proxy>::Event,
        idx: &usize,
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!("* Window ({idx}) can draw now (frame request) <- compositor");

        if let Some(window) = state.windows.get_mut(*idx) {
            if let Some(buffer) = window
                .buffers
                .iter_mut()
                .filter(|buffer| (!buffer.destroy && !buffer.used))
                .next()
            {
                buffer.used = true; // FIXME: for some reason if i do this some windows lost
                // their buffer, if i don't nothing wrong seems to happen
                window.surface.attach(Some(&buffer.data), 0, 0);
                window.surface.damage(0, 0, window.width, window.height);
                window.surface.commit();
            } else {
                println!("!! Warning: No available buffer for window {}", idx);
            }

            let frame = window.surface.frame(&qhandle, *idx);
            window.frame = frame;
        }
    }
}
