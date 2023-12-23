use std::net::SocketAddr;
use std::{collections::HashMap, error::Error};

use std::io;
use std::time::Duration;

use mio::net::TcpStream;
use mio::{net::TcpListener, Events, Interest, Poll, Token};

pub struct Client {
    socket: TcpStream,
    addr: SocketAddr,
}

impl Client {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Self {
        Client { socket, addr }
    }
}

pub struct WebSocketServer {
    clients: HashMap<Token, Client>,
    socket: mio::net::TcpListener,
    token: Token,
    token_counter: usize,
}

impl WebSocketServer {
    pub fn new(addr: SocketAddr) -> Self {
        WebSocketServer {
            clients: HashMap::new(),
            socket: TcpListener::bind(addr).unwrap(),
            token: Token(0),
            token_counter: 1,
        }
    }

    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        let mut poller = Poll::new()?;
        let mut events = Events::with_capacity(128);

        poller
            .registry()
            .register(&mut self.socket, self.token, Interest::READABLE)?;

        loop {
            poller.poll(&mut events, Some(Duration::from_millis(100)))?;

            for event in events.iter() {
                match event.token() {
                    t if t == self.token => loop {
                        match self.socket.accept() {
                            Ok((conn, addr)) => {
                                println!("connection established with {}", addr);

                                let client = Client::new(conn, addr);
                                self.clients.insert(Token(self.token_counter), client);
                                self.token_counter += 1;
                            }
                            Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => break,
                            Err(err) => return Err(Box::new(err)),
                        }
                    },
                    Token(_) => unreachable!(),
                }
            }
        }
    }
}
