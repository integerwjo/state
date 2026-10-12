// Implementing the http client

use crate::future::{Future, PollState};
use std::io::{ErrorKind, Read, Write};

pub struct Http;

impl Http {
    // returns an object that implements the future trait that outputs a string when it is resolved
    // The string returned will be the response from the server
    pub fn get(path: &'static str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
                 Host: localhost\r\n\
                 Connection: close\r\n\
                 \r\n"
    )
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    write_buffer: Vec<u8>,
    write_offset: usize,
    read_buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    fn new(path: &'static str) -> Self {
        let req = get_req(path);
        Self {
            stream: None,
            write_buffer: req.into_bytes(),
            write_offset: 0,
            read_buffer: vec![],
            path: path.to_string(),
        }
    }

    fn init_stream(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let stream = mio::net::TcpStream::from_std(stream);
        self.stream = Some(stream);
    }

    fn write_request(&mut self) -> PollState<String> {
        let stream = match self.stream.as_mut() {
            Some(s) => s,
            None => return PollState::NotReady,
        };

        loop {
            if self.write_offset == self.write_buffer.len() {
                // Request fully written, move to reading
                return PollState::NotReady;
            }

            match stream.write(&self.write_buffer[self.write_offset..]) {
                Ok(n) => {
                    self.write_offset += n;
                    if self.write_offset == self.write_buffer.len() {
                        return PollState::NotReady;
                    }
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    return PollState::NotReady;
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => {
                    panic!("{e:?}");
                }
            }
        }
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.init_stream();
        }

        // Ensure request is fully written before reading
        if self.write_offset < self.write_buffer.len() {
            return self.write_request();
        }

        let mut buff = vec![0u8; 4096];

        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.read_buffer);
                    break PollState::Ready(s.to_string());
                }
                Ok(n) => {
                    self.read_buffer.extend(&buff[..n]);
                    continue;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break PollState::NotReady,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
