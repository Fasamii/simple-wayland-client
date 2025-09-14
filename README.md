# Simple Wayland Client
A minimal library for creating Wayland windows with ease.
This library provides a straightforward interface to connect to a Wayland compositor and create windows without diving into low-level details.
## Usage
1.  Create a Client
First thing you should do while using this library is create Client instance by:
```rust
use simple_wayland_client::Client;
fn main() {
    let client = Client::new().unwrap();
}
```
2. Create a Window
Then after you established connection with compositor you can create window by:
```rust
let window_index = client.create_window("name", "app-id").unwrap();
```
Windows are stored internally in a Vec. To access a specific one, use the returned index:
```rust
let window = client.globals.windows.get(window_index).unwrap();
```
