use judy::JudyL;
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::{io::{self, prelude::*, BufReader, BufWriter}};

fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
    match conn {
        Ok(val) => Some(val),
        Err(error) => {
            eprintln!("Incoming connection failed: {}", error);
            None
        }
    }
}

fn main() {
    let mut tree = JudyL::new();
    let listener = LocalSocketListener::bind("../comms.sock").unwrap();
    for conn in listener.incoming().filter_map(handle_error) {
        let mut reader = BufReader::new(conn);
        let mut buffer = [0; 17];
        reader.read_exact(&mut buffer);
        dbg!(buffer);
        let mut resp: u64 = 0xffffffffffffffffu64;
        match buffer[0] as char {
            'G' => {
                let k = u64::from_be_bytes(buffer[1..9].try_into().expect("8, but not really"));
                match tree.get(k) {
                    Some(res) => {resp = res;},
                    None => {}
                }
                
            },
            'I' => {
                let k = u64::from_be_bytes(buffer[1..9].try_into().expect("8, but not really"));
                let v = u64::from_be_bytes(buffer[9..17].try_into().expect("8, but not really"));
                tree.insert(k, v);
                continue;
            },
            _ => {}
        }
        let mut conn = BufWriter::new(reader.into_inner());
        conn.write_all(resp.to_be_bytes().as_slice()).expect("Failed to write");
    }
}