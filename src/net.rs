use crate::http::{HttpRequest, HttpResponse, Method};
use std::collections::HashMap;
use mio::{Token, Events, Poll, Interest, Registry};
use std::net::{ToSocketAddrs, SocketAddr};
use mio::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use crate::parser::Parser;

pub struct HttpServer {
    map: HashMap<(Method, String), Box<dyn 'static + Fn(&HttpRequest) -> HttpResponse>>,
    r_map: Vec<(Method, Box<dyn 'static + Fn(&str) -> bool>, Box<dyn 'static + Fn(&HttpRequest) -> HttpResponse>)>,
    default: Option<Box<dyn 'static + Fn(&HttpRequest) -> HttpResponse>>,
    clients: HashMap<Token, Client>,
}

struct Client {
    stream: TcpStream,
    address: SocketAddr,
    parser: Parser,
    token: Token,
    cache: Option<HttpResponse>, // TODO: Replace with boxed trait for streams and large responses
}

impl HttpServer {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            r_map: vec![],
            default: None,
            clients: Default::default(),
        }
    }

    pub fn register_handler<F>(&mut self, method: Method, route: String, handler: F)
        where F: 'static + Fn(&HttpRequest) -> HttpResponse
    {
        self.map.insert((method, route), Box::new(handler));
    }

    pub fn register_matching_handler<M, F>(&mut self, method: Method, route: M, handler: F)
        where M: 'static + Fn(&str) -> bool, F: 'static + Fn(&HttpRequest) -> HttpResponse
    {
        self.r_map.push((method, Box::new(route), Box::new(handler)));
    }

    pub fn register_default<F>(&mut self, handler: F)
        where F: 'static + Fn(&HttpRequest) -> HttpResponse
    {
        self.default = Some(Box::new(handler));
    }

    pub fn run(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        let mut poll: Poll = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        let mut listener = TcpListener::bind(addr)?;
        let mut server_token = Token(0);
        poll.registry().register(&mut listener, server_token, Interest::READABLE)?;
        let mut last_token = Token(server_token.0 + 1);
        loop {
            poll.poll(&mut events, None);
            for event in events.iter() {
                let mut remove = false;
                if event.token() == server_token {
                    match listener.accept() {
                        Ok((connection, address)) => {
                            self.handle_connection_sync(poll.registry(), connection, address, last_token.add_one())?;
                        }
                        Err(ref err) if would_block(err) => continue,
                        x => return x.map(|_| ()) // will always be err
                    }
                } else {
                    remove = if event.is_readable() {
                        if let Ok(remove) = self.parse_client(poll.registry(), event.token()) {
                            remove
                        } else {
                            true
                        }
                    } else if event.is_writable() {
                        if let Ok(remove) = self.send_response(poll.registry(), event.token()) {
                            remove
                        } else {
                            true
                        }
                    } else {
                        eprintln!("no read no write?");
                        true
                    };
                }
                if remove {
                    if let Some(client) = self.clients.get_mut(&event.token()) {
                        poll.registry().deregister(&mut client.stream);
                    }
                    self.clients.remove(&event.token());
                }
            }
        }
        Ok(())
    }

    pub fn handle_connection_sync(&mut self, registry: &Registry, mut connection: TcpStream, address: SocketAddr, token: Token) -> std::io::Result<()> {
        registry.register(&mut connection, token, Interest::READABLE)?;
        let mut client = Client::new(connection, address, token);
        self.clients.insert(token, client);
        Ok(())
    }

    pub fn parse_client(&mut self, registry: &Registry, token: Token) -> std::io::Result<bool> {
        Ok(if let Some(client) = self.clients.get_mut(&token) {
            let mut buffer = [0u8; 2048];
            let mut read = 0usize;
            while {
                read = match client.stream.read(&mut buffer) {
                    Ok(r) => r,
                    Err(ref err) if would_block(err) => 0,
                    x => return x.map(|_| false) // will always be err
                };
                read != 0
            } {
                if client.parser.parse(&buffer[..read]) {
                    registry.reregister(&mut client.stream, client.token, Interest::WRITABLE);
                    let request = &client.parser.request;
                    let endpoint = (request.method.clone(), request.path.clone());
                    if let Some(handler) = self.map.get(&endpoint) {
                        client.cache = Some(handler(request));
                    } else {
                        for (method, matcher, handler) in self.r_map.iter() {
                            if *method != endpoint.0 || !matcher(endpoint.1.as_str()) {
                                continue;
                            }
                            client.cache = Some(handler(request));
                        }
                        if let Some(handler) = &self.default {
                            client.cache = Some(handler(request));
                        } else {
                            unreachable!()
                        }
                    }
                }
            } // Magical do-while-do look :D
            false // FIXME: this is for later error checking in case the request contains invalid data
        } else {
            false
        })
    }

    pub fn send_response(&mut self, registry: &Registry, token: Token) -> std::io::Result<bool> {
        Ok(if let Some(client) = self.clients.get_mut(&token) {
            // TODO: idc if this might block i just wanna test this fix this tomorrow
            let res = if let Some(response) = &client.cache {
                let r_code = response.code.get();
                client.stream.write_all(format!("HTTP/1.1 {} {}\r\n", r_code.0, r_code.1).as_bytes())?;
                for header in &response.header {
                    client.stream.write_all(format!("{}: {}\r\n", header.0, header.1).as_bytes())?;
                }
                client.stream.write("\r\n".as_bytes())?;
                if response.len > 0 {
                    client.stream.write_all(&response.buffer[..response.len])?;
                }
                client.stream.flush()?;
                true // false
            } else {
                true
            };
            res
        } else {
            false
        })
    }
}

impl Client {
    pub fn new(connection: TcpStream, address: SocketAddr, token: Token) -> Self {
        Self {
            stream: connection,
            address,
            parser: Parser::new(),
            token,
            cache: None,
        }
    }
}

#[inline(always)]
fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

trait AddOne {
    fn add_one(&mut self) -> Self;
}

impl AddOne for Token {
    fn add_one(&mut self) -> Self {
        let tmp = *self; // Due to Token implementing Copy this acts like cloning it
        self.0 += 1;
        tmp
    }
}
