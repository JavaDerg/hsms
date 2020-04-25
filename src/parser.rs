use crate::http::{Method, MAX_CONTENT_SIZE, HttpRequest};
use std::rc::Rc;
use std::collections::HashMap;

pub struct Parser {
    state: State,
    buffer: [u8; MAX_CONTENT_SIZE],
    buf_pos: usize,
    content: Option<usize>,
    pub request: HttpRequest, // TODO: make a getter function and stuff
}

enum State {
    Method(String),
    Path(String),
    Protocol(String),
    Version(String),
    Header(Option<HashMap<String, String>>, String),
    Content,
    Done
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Method(String::with_capacity(8)),
            buffer: [0u8; MAX_CONTENT_SIZE],
            buf_pos: 0,
            content: None,
            request: HttpRequest {
                method: Method::None,
                path: String::new(),
                protocol: String::new(),
                version: String::new(),
                headers: Default::default()
            }
        }
    }

    pub fn parse(&mut self, bytes: &[u8]) -> bool {
        let mut call_next = None;
        if let Some(s) = match &mut self.state {
            State::Method(buffer) => {
                let mut ret = None;
                for (b, i) in bytes.iter().zip(0usize..) {
                    if *b == b' ' {
                        self.request.method = crate::http::Method::parse(buffer.as_str());
                        ret = Some(State::Path(String::with_capacity(512)));
                        call_next = Some(i + 1);
                        break;
                    }
                    buffer.push(*b as char); // Assuming ascii here
                }
                ret
            }
            State::Path(buffer) => {
                let mut ret = None;
                for (b, i) in bytes.iter().zip(0usize..) {
                    if *b == b' ' {
                        self.request.path = buffer.clone();
                        ret = Some(State::Protocol(String::with_capacity(4)));
                        call_next = Some(i + 1);
                        break;
                    }
                    buffer.push(*b as char); // Assuming ascii here
                }
                ret
            }
            State::Protocol(buffer) => {
                let mut ret = None;
                for (b, i) in bytes.iter().zip(0usize..) {
                    if *b == b'/' {
                        self.request.protocol = buffer.clone();
                        ret = Some(State::Version(String::with_capacity(3)));
                        call_next = Some(i + 1);
                        break;
                    }
                    buffer.push(*b as char); // Assuming ascii here
                }
                ret
            }
            State::Version(buffer) => {
                let mut ret = None;
                for (b, i) in bytes.iter().zip(0usize..) {
                    match *b {
                        b'\n' => {
                            self.request.version = buffer.clone();
                            ret = Some(State::Header(Some(HashMap::with_capacity(16)), String::with_capacity(256)));
                            call_next = Some(i + 1);
                            break;
                        }
                        b'\r' => continue,
                        _ => buffer.push(*b as char) // Assuming ascii here
                    }
                }
                ret
            }
            State::Header(header, buffer) => {
                let mut ret = None;
                for (b, i) in bytes.iter().zip(0usize..) {
                    match *b {
                        b'\n' => {
                            if buffer.is_empty() {
                                self.request.headers = if let Some(map) = header {
                                    let mut tmp = HashMap::new();
                                    std::mem::swap(map, &mut tmp);
                                    tmp
                                } else { unreachable!() };
                                ret = Some(State::Content);
                                call_next = Some(i + 1);
                                break;
                            } else {
                                if let Some(pos) = buffer.find(':') {
                                    let (name, value) = buffer.split_at(pos);
                                    if let Some(map) = header {
                                        map.insert(name.to_string(), (&value[1..]).trim().to_string())
                                    } else { unreachable!() };
                                    if value == "Content-Length" {
                                        if let Ok(size) = value.trim().parse::<usize>() {
                                            self.content = Some(size);
                                        }
                                    }
                                } // else => invalid header => ignore
                                buffer.clear();
                            }
                        }
                        b'\r' => continue,
                        x => buffer.push(x as char) // Assuming ascii here
                    }
                }
                ret
            }
            State::Content => {
                if let Some(mut max_len) = self.content {
                    if max_len > self.buffer.len() {
                        // Maximum buffer size exceeded, ignore additional content
                        max_len = self.buffer.len();
                    }
                    let len = if bytes.len() > max_len - self.buf_pos {
                        max_len - self.buf_pos
                    } else {
                        bytes.len()
                    };
                    (&mut self.buffer[self.buf_pos..(self.buf_pos + len)]).clone_from_slice(&bytes[..len]);
                    self.buf_pos += len;
                    if self.buf_pos == max_len {
                        Some(State::Done)
                    } else {
                        None
                    }
                } else {
                    Some(State::Done)
                }
            },
            State::Done => unreachable!()
        } {
            self.state = s;
        }
        if let Some(n) = call_next {
            let _ = self.parse(&bytes[n..]);
        }
        if let State::Done = self.state {
            true
        } else {
            false
        }
    }
}
