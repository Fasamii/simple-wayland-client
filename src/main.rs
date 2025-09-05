pub mod wl_client;

fn main() {
    let mut client = wl_client::Client::new().unwrap();
    let res = client.create_surface();

    if let Err(res) = res {
        println!("{res}");
        println!("{:?}", res.kind());
        std::process::exit(1);
    }

    loop {
        client.queue.blocking_dispatch(&mut client.state).unwrap();
    }
}
