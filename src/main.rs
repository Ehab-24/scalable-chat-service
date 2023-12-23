use std::error::Error;
use std::net::SocketAddr;

extern crate mio;

mod server;

fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
    server::WebSocketServer::new(addr).listen()?;

    Ok(())
}
