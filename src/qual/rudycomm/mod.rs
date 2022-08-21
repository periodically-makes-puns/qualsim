use std::net::TcpStream;
use std::io::{Read, Write};
pub struct SubrudyCommunicator {
    stream: TcpStream
}

impl SubrudyCommunicator {
    pub fn new() -> SubrudyCommunicator {
        SubrudyCommunicator {
            stream: TcpStream::connect("127.0.0.1:39570").unwrap()
        }
    }

    pub fn get(&mut self, key: u64) -> Option<u64> {
        let sent: [u8; 8] = key.to_le_bytes();
        self.stream.write(&sent);
        let mut buf: [u8; 8] = [0; 8];
        self.stream.read(&mut buf);
        match u64::from_le_bytes(buf) {
            0xffffffffffffffffu64 => None,
            x => Some(x)
        }
    }

    pub fn insert(&mut self, key: u64, value: u64) {
        let key_bytes: [u8; 8] = key.to_le_bytes();
        let value_bytes: [u8; 8] = value.to_le_bytes();
        let sent: [u8; 16] = [key_bytes, value_bytes].concat().try_into().expect("ok");
        self.stream.write(&sent);
    }
}