extern crate rustc_serialize;
extern crate sha1;

use rustc_serialize::base64::{ToBase64, STANDARD};
use sha1::{Digest, Sha1};

use std::net::SocketAddr;
use std::{collections::HashMap, error::Error};

use std::io::{self, Read};
use std::time::Duration;

use mio::{net::TcpListener, net::TcpStream, Events, Interest, Poll, Token};

use http_muncher::{Parser, ParserHandler};

struct HttpHandler;
impl ParserHandler for HttpHandler {}

pub struct WebSocketClient {
    socket: TcpStream,
    addr: SocketAddr,
    http_parser: Parser,
}

impl WebSocketClient {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> Self {
        WebSocketClient {
            socket,
            addr,
            http_parser: Parser::request(),
        }
    }

    pub fn read(&mut self) {
        let mut buf = Vec::<u8>::new();
        loop {
            match self.socket.read_to_end(&mut buf) {
                Ok(size) => {
                    println!("read {} bytes", size);
                    self.http_parser.parse(&mut HttpHandler {}, &buf[0..size]);
                    if self.http_parser.is_upgrade() {
                        println!("upgrading {}", self.addr);
                        break;
                    }
                }
                Err(_) => {
                    println!("Error while reading socket");
                    return;
                }
            }
        }
    }
}

pub struct WebSocketServer {
    clients: HashMap<Token, WebSocketClient>,
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

                                let client = WebSocketClient::new(conn, addr);
                                self.clients.insert(Token(self.token_counter), client);
                                self.token_counter += 1;
                            }
                            Err(ref err) if would_block(err) => break,
                            Err(err) => return Err(Box::new(err)),
                        }
                    },

                    token => match self.clients.get_mut(&token) {
                        Some(client) => {
                            client.read();
                            poller.registry().reregister(
                                &mut client.socket,
                                token,
                                Interest::READABLE,
                            )?;
                        }
                        None => println!("unknown token: {:?}", token),
                    },
                }
            }
        }
    }
}

fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}
